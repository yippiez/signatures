"""Signatures that span several lines, plus type annotations."""

from typing import Dict, List, Optional


def configure(
    host: str,
    port: int = 8080,
    *args,
    timeout: float = 30.0,
    retries: int = 3,
    **kwargs,
) -> Dict[str, int]:
    return {}


async def gather(
    items: List[str],
    limit: Optional[int] = None,
) -> List[str]:
    return items


class Server(
    BaseHandler,
    metaclass=Meta,
):
    HOST: str = "0.0.0.0"

    def run(self) -> None:
        pass
