/**
 * A realistic HTTP client module with classes, exports, and constants.
 * function notASignature() -- inside block comment, should be ignored
 */

import { EventEmitter } from 'events';

export const DEFAULT_TIMEOUT = 5000;
export const BASE_URL = "https://api.example.com";
const MAX_RETRIES = 3;

export class HttpError extends Error {
  constructor(status, message) {
    super(message);
    this.status = status;
    this.name = "HttpError";
  }

  isClientError() {
    return this.status >= 400 && this.status < 500;
  }

  isServerError() {
    return this.status >= 500;
  }

  toString() {
    return `${this.name}: ${this.status} ${this.message}`;
  }
}

export class HttpClient extends EventEmitter {
  constructor(baseUrl, options) {
    super();
    this.baseUrl = baseUrl || BASE_URL;
    this.timeout = options?.timeout || DEFAULT_TIMEOUT;
    this._interceptors = [];
  }

  get defaultHeaders() {
    return {
      'Content-Type': 'application/json',
      'Accept': 'application/json',
    };
  }

  set authToken(token) {
    this._token = token;
  }

  addInterceptor(fn) {
    this._interceptors.push(fn);
  }

  async get(path, params) {
    return this._request('GET', path, null, params);
  }

  async post(path, body) {
    return this._request('POST', path, body);
  }

  async put(path, body) {
    return this._request('PUT', path, body);
  }

  async delete(path) {
    return this._request('DELETE', path);
  }

  async _request(method, path, body, params) {
    const url = new URL(this.baseUrl + path);
    if (params) {
      Object.entries(params).forEach(([k, v]) => url.searchParams.set(k, v));
    }
    let attempt = 0;
    while (attempt < MAX_RETRIES) {
      try {
        const res = await fetch(url, {
          method,
          headers: this.defaultHeaders,
          body: body ? JSON.stringify(body) : undefined,
        });
        if (!res.ok) throw new HttpError(res.status, await res.text());
        return await res.json();
      } catch (err) {
        attempt++;
        if (attempt >= MAX_RETRIES) throw err;
      }
    }
  }
}

export function createClient(baseUrl, options) {
  return new HttpClient(baseUrl, options);
}

export default HttpClient;
