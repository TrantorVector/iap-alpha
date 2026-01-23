import "@testing-library/jest-dom";
import { vi, afterEach } from "vitest";

// Mock IntersectionObserver which isn't available in JSDOM
class MockIntersectionObserver {
  observe = vi.fn();
  disconnect = vi.fn();
  unobserve = vi.fn();
}

Object.defineProperty(window, "IntersectionObserver", {
  writable: true,
  configurable: true,
  value: MockIntersectionObserver,
});

// Clean up after each test
afterEach(() => {
  vi.clearAllMocks();
});
