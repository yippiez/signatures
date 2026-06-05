"""
A realistic web-service-style module: config, models, service layer, CLI entry point.
Mirrors patterns found in production Django/FastAPI codebases.
"""

from __future__ import annotations

import logging
import os
from dataclasses import dataclass, field
from typing import Any, Dict, List, Optional, Sequence, Tuple

logger = logging.getLogger(__name__)

# ---------------------------------------------------------------------------
# Module-level constants
# ---------------------------------------------------------------------------
DEFAULT_TIMEOUT: int = 30
MAX_RETRIES: int = 3
BASE_URL: str = "https://api.example.com/v2"
SUPPORTED_FORMATS: Tuple[str, ...] = ("json", "msgpack", "protobuf")
_internal_version = "1.0.0-dev"  # lowercase: not a constant


# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------


@dataclass
class DatabaseConfig:
    """Database connection settings."""

    HOST: str = "localhost"
    PORT: int = 5432
    NAME: str = "app_db"
    USER: str = "postgres"
    PASSWORD: str = ""
    pool_size: int = 10  # lowercase: not a constant

    def dsn(self) -> str:
        return f"postgresql://{self.USER}:{self.PASSWORD}@{self.HOST}:{self.PORT}/{self.NAME}"

    @classmethod
    def from_env(cls) -> "DatabaseConfig":
        return cls(
            HOST=os.getenv("DB_HOST", "localhost"),
            PORT=int(os.getenv("DB_PORT", "5432")),
            NAME=os.getenv("DB_NAME", "app_db"),
            USER=os.getenv("DB_USER", "postgres"),
            PASSWORD=os.getenv("DB_PASSWORD", ""),
        )


@dataclass
class AppConfig:
    """Top-level application configuration."""

    DEBUG: bool = False
    SECRET_KEY: str = "change-me"
    ALLOWED_HOSTS: List[str] = field(default_factory=list)
    db: DatabaseConfig = field(default_factory=DatabaseConfig)

    def validate(self) -> None:
        if not self.SECRET_KEY or self.SECRET_KEY == "change-me":
            raise ValueError("SECRET_KEY must be set in production")

    def is_production(self) -> bool:
        return not self.DEBUG


# ---------------------------------------------------------------------------
# Domain models
# ---------------------------------------------------------------------------


class User:
    """Represents an authenticated user."""

    ROLES: Tuple[str, ...] = ("viewer", "editor", "admin")

    def __init__(self, user_id: int, username: str, role: str = "viewer") -> None:
        self.user_id = user_id
        self.username = username
        self.role = role
        self._token: Optional[str] = None

    def __repr__(self) -> str:
        return f"User(id={self.user_id}, username={self.username!r})"

    def has_permission(self, action: str) -> bool:
        permissions = {
            "viewer": {"read"},
            "editor": {"read", "write"},
            "admin": {"read", "write", "delete", "manage"},
        }
        return action in permissions.get(self.role, set())

    @property
    def token(self) -> Optional[str]:
        return self._token

    @token.setter
    def token(self, value: str) -> None:
        if not isinstance(value, str) or len(value) < 16:
            raise ValueError("Token must be a string of at least 16 characters")
        self._token = value

    @classmethod
    def anonymous(cls) -> "User":
        return cls(user_id=0, username="anonymous", role="viewer")


class Article:
    """Represents a content article."""

    STATUS_DRAFT: str = "draft"
    STATUS_PUBLISHED: str = "published"
    STATUS_ARCHIVED: str = "archived"

    def __init__(
        self,
        article_id: int,
        title: str,
        body: str,
        author: User,
        tags: Optional[List[str]] = None,
    ) -> None:
        self.article_id = article_id
        self.title = title
        self.body = body
        self.author = author
        self.tags = tags or []
        self.status = self.STATUS_DRAFT

    def publish(self) -> None:
        if self.status != self.STATUS_DRAFT:
            raise RuntimeError("Only draft articles can be published")
        self.status = self.STATUS_PUBLISHED

    def archive(self) -> None:
        self.status = self.STATUS_ARCHIVED

    def word_count(self) -> int:
        return len(self.body.split())

    def summary(self, max_words: int = 50) -> str:
        words = self.body.split()
        return " ".join(words[:max_words]) + ("..." if len(words) > max_words else "")


# ---------------------------------------------------------------------------
# Service layer
# ---------------------------------------------------------------------------


class ArticleService:
    """Business logic for article management."""

    PAGE_SIZE: int = 20

    def __init__(self, config: AppConfig) -> None:
        self._config = config
        self._store: Dict[int, Article] = {}
        self._next_id: int = 1

    def create(self, title: str, body: str, author: User, tags: Optional[List[str]] = None) -> Article:
        article = Article(self._next_id, title, body, author, tags)
        self._store[self._next_id] = article
        self._next_id += 1
        logger.info("Created article %d by %s", article.article_id, author.username)
        return article

    def get(self, article_id: int) -> Article:
        try:
            return self._store[article_id]
        except KeyError:
            raise LookupError(f"Article {article_id} not found")

    def list_published(self, page: int = 1) -> List[Article]:
        published = [a for a in self._store.values() if a.status == Article.STATUS_PUBLISHED]
        start = (page - 1) * self.PAGE_SIZE
        return published[start : start + self.PAGE_SIZE]

    def search(self, query: str, tags: Optional[Sequence[str]] = None) -> List[Article]:
        results = []
        for article in self._store.values():
            if query.lower() in article.title.lower() or query.lower() in article.body.lower():
                if tags is None or any(t in article.tags for t in tags):
                    results.append(article)
        return results

    def delete(self, article_id: int, requestor: User) -> None:
        if not requestor.has_permission("delete"):
            raise PermissionError(f"{requestor.username} cannot delete articles")
        article = self.get(article_id)
        del self._store[article.article_id]


# ---------------------------------------------------------------------------
# CLI helpers
# ---------------------------------------------------------------------------


def parse_args(argv: Optional[List[str]] = None) -> Dict[str, Any]:
    import argparse

    parser = argparse.ArgumentParser(description="Article management CLI")
    parser.add_argument("command", choices=["create", "list", "search", "delete"])
    parser.add_argument("--title", help="Article title")
    parser.add_argument("--body", help="Article body text")
    parser.add_argument("--author-id", type=int, default=1)
    parser.add_argument("--page", type=int, default=1)
    parser.add_argument("--query", help="Search query")
    ns = parser.parse_args(argv)
    return vars(ns)


def setup_logging(debug: bool = False) -> None:
    level = logging.DEBUG if debug else logging.INFO
    logging.basicConfig(level=level, format="%(levelname)s %(name)s: %(message)s")


def main(argv: Optional[List[str]] = None) -> int:
    args = parse_args(argv)
    config = AppConfig(DEBUG=bool(os.getenv("DEBUG")))
    setup_logging(config.DEBUG)
    service = ArticleService(config)
    cmd = args["command"]
    if cmd == "list":
        for a in service.list_published(page=args["page"]):
            print(a)
    elif cmd == "search":
        for a in service.search(args.get("query", "")):
            print(a)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
