/**
 * Real-world HTTP API client module.
 * Covers: exported constants, interfaces, classes, async methods, getters.
 */

export const BASE_URL: string = "https://api.example.com/v1";
export const DEFAULT_TIMEOUT_MS: number = 5000;
export const MAX_RETRIES: number = 3;

export interface RequestOptions {
  timeout?: number;
  retries?: number;
  headers?: Record<string, string>;
}

export interface ApiResponse<T> {
  data: T;
  status: number;
  message: string;
}

export interface UserProfile {
  id: string;
  username: string;
  email: string;
  createdAt: Date;
}

export interface PaginatedResult<T> {
  items: T[];
  total: number;
  page: number;
  pageSize: number;
}

export class ApiError extends Error {
  constructor(
    public readonly statusCode: number,
    public readonly body: string,
    message: string,
  ) {
    super(message);
    this.name = "ApiError";
  }

  get isClientError(): boolean {
    return this.statusCode >= 400 && this.statusCode < 500;
  }

  get isServerError(): boolean {
    return this.statusCode >= 500;
  }
}

export class HttpClient {
  private readonly baseUrl: string;
  private defaultHeaders: Record<string, string>;

  constructor(baseUrl: string = BASE_URL, token?: string) {
    this.baseUrl = baseUrl;
    this.defaultHeaders = { "Content-Type": "application/json" };
    if (token) {
      this.defaultHeaders["Authorization"] = `Bearer ${token}`;
    }
  }

  get endpoint(): string {
    return this.baseUrl;
  }

  async fetchJson(path: string, opts?: RequestOptions): Promise<unknown> {
    const url = this.baseUrl + path;
    const resp = await fetch(url, { headers: this.defaultHeaders });
    if (!resp.ok) {
      throw new ApiError(resp.status, await resp.text(), "fetch failed");
    }
    return resp.json();
  }

  async sendJson(path: string, body: unknown, opts?: RequestOptions): Promise<unknown> {
    const resp = await fetch(this.baseUrl + path, {
      method: "POST",
      headers: this.defaultHeaders,
      body: JSON.stringify(body),
    });
    if (!resp.ok) {
      throw new ApiError(resp.status, await resp.text(), "send failed");
    }
    return resp.json();
  }

  async removeResource(path: string): Promise<void> {
    const resp = await fetch(this.baseUrl + path, { method: "DELETE" });
    if (!resp.ok) {
      throw new ApiError(resp.status, await resp.text(), "remove failed");
    }
  }
}

export class UserService {
  private client: HttpClient;

  constructor(client: HttpClient) {
    this.client = client;
  }

  async getUser(id: string): Promise<UserProfile> {
    return this.client.fetchJson(`/users/${id}`) as Promise<UserProfile>;
  }

  async createUser(profile: Omit<UserProfile, "id" | "createdAt">): Promise<UserProfile> {
    return this.client.sendJson("/users", profile) as Promise<UserProfile>;
  }

  async removeUser(id: string): Promise<void> {
    await this.client.removeResource(`/users/${id}`);
  }
}

export function buildClient(token: string): HttpClient {
  return new HttpClient(BASE_URL, token);
}

export async function fetchUserById(id: string, token: string): Promise<UserProfile> {
  const svc = new UserService(buildClient(token));
  return svc.getUser(id);
}
