/**
 * ThreadList Component Tests - Merge Button State Propagation
 *
 * Tests that ThreadList correctly passes mergeButtonState from the
 * loops.list API response to LoopActions component for each worktree loop.
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, within } from "@testing-library/react";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { MemoryRouter } from "react-router-dom";

// Mock tRPC hooks with inline data (vi.mock is hoisted)
vi.mock("@/trpc", () => {
  // Create a mock mutation function - using inline noop functions
  const noop = () => {};
  const createMockMutation = () => ({
    mutate: noop,
    mutateAsync: noop,
    isPending: false,
    isError: false,
    error: null,
  });

  return {
    trpc: {
      task: {
        list: {
          useQuery: () => ({
            data: [
              {
                id: "task-001",
                title: "Test task",
                status: "open",
                priority: 2,
                createdAt: new Date().toISOString(),
                updatedAt: new Date().toISOString(),
              },
            ],
            isLoading: false,
            isError: false,
            error: null,
            isFetching: false,
            refetch: noop,
          }),
        },
        get: {
          useQuery: () => ({
            data: null,
            isLoading: false,
          }),
        },
        run: { useMutation: () => createMockMutation() },
        cancel: { useMutation: () => createMockMutation() },
        retry: { useMutation: () => createMockMutation() },
        close: { useMutation: () => createMockMutation() },
        archive: { useMutation: () => createMockMutation() },
        update: { useMutation: () => createMockMutation() },
        delete: { useMutation: () => createMockMutation() },
        executionStatus: {
          useQuery: () => ({
            data: { isQueued: false },
            isLoading: false,
          }),
        },
      },
      loops: {
        list: {
          useQuery: () => ({
            data: [
              {
                id: "loop-worktree-001",
                status: "queued",
                prompt: "Implement feature A",
                location: ".worktrees/feature-a",
                workspaceRoot: "/path/to/.worktrees/feature-a",
                repoRoot: "/path/to/repo",
                pid: 12345,
                mergeButtonState: { state: "active" },
              },
              {
                id: "loop-worktree-002",
                status: "queued",
                prompt: "Implement feature B",
                location: ".worktrees/feature-b",
                workspaceRoot: "/path/to/.worktrees/feature-b",
                repoRoot: "/path/to/repo",
                pid: 12346,
                mergeButtonState: { state: "blocked", reason: "Primary loop is running: Building core module" },
              },
              {
                id: "loop-primary",
                status: "running",
                prompt: "Building core module",
                location: "(in-place)",
                workspaceRoot: "/path/to/repo",
                repoRoot: "/path/to/repo",
                pid: 12340,
              },
            ],
            isLoading: false,
            isError: false,
            error: null,
          }),
        },
        stop: { useMutation: () => createMockMutation() },
        retry: { useMutation: () => createMockMutation() },
        merge: { useMutation: () => createMockMutation() },
        discard: { useMutation: () => createMockMutation() },
      },
      useUtils: () => ({
        loops: { list: { invalidate: noop } },
        task: { list: { invalidate: noop } },
      }),
    },
  };
});

// Mock the hooks
vi.mock("@/hooks", () => ({
  useNotifications: vi.fn(() => ({
    permission: "default",
    enabled: false,
    requestPermission: vi.fn(),
    setEnabled: vi.fn(),
    checkTaskStatusChanges: vi.fn(),
    isSupported: true,
  })),
  useKeyboardShortcuts: vi.fn(() => ({
    isTaskFocused: vi.fn(() => false),
  })),
}));

import { ThreadList } from "./ThreadList";

function createTestWrapper() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });
  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>{children}</MemoryRouter>
    </QueryClientProvider>
  );
}

describe("ThreadList merge button state propagation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("Active Loops section", () => {
    it("renders merge button as enabled when mergeButtonState is active", () => {
      // Given: ThreadList with loops data containing a worktree loop with active merge state
      render(<ThreadList />, { wrapper: createTestWrapper() });

      // Then: The merge button for the active-state loop should be enabled
      // Navigate to the card container (rounded-lg border bg-card p-3)
      const activeLoopsContainer = screen
        .getByRole("heading", { name: /active loops/i })
        .closest(".rounded-lg") as HTMLElement;

      // Find the loop row for feature-a (which has active state)
      const featureARow = within(activeLoopsContainer)
        .getByText(/implement feature a/i)
        .closest(".bg-muted\\/50") as HTMLElement;

      const mergeButton = within(featureARow).getByRole("button", { name: /merge now/i });
      expect(mergeButton).toBeEnabled();
      expect(mergeButton).not.toHaveClass("opacity-50");
    });

    it("renders merge button as blocked when mergeButtonState is blocked", () => {
      // Given: ThreadList with loops data containing a worktree loop with blocked merge state
      render(<ThreadList />, { wrapper: createTestWrapper() });

      // Then: The merge button for the blocked-state loop should be disabled
      const activeLoopsContainer = screen
        .getByRole("heading", { name: /active loops/i })
        .closest(".rounded-lg") as HTMLElement;

      // Find the loop row for feature-b (which has blocked state)
      const featureBRow = within(activeLoopsContainer)
        .getByText(/implement feature b/i)
        .closest(".bg-muted\\/50") as HTMLElement;

      const mergeButton = within(featureBRow).getByRole("button", { name: /merge now/i });
      expect(mergeButton).toBeDisabled();
      expect(mergeButton).toHaveClass("opacity-50");
    });

    it("shows blocked reason in tooltip when mergeButtonState is blocked", () => {
      // Given: ThreadList with loops data containing a worktree loop with blocked merge state and reason
      render(<ThreadList />, { wrapper: createTestWrapper() });

      // Then: The blocked merge button should show the reason in its tooltip
      const activeLoopsContainer = screen
        .getByRole("heading", { name: /active loops/i })
        .closest(".rounded-lg") as HTMLElement;

      // Find the loop row for feature-b (which has blocked state with reason)
      const featureBRow = within(activeLoopsContainer)
        .getByText(/implement feature b/i)
        .closest(".bg-muted\\/50") as HTMLElement;

      const mergeButton = within(featureBRow).getByRole("button", { name: /merge now/i });
      expect(mergeButton).toHaveAttribute("title", expect.stringContaining("Primary loop is running"));
    });

    it("does not render merge button for primary loop (in-place)", () => {
      // Given: ThreadList with loops data containing a primary loop (in-place)
      render(<ThreadList />, { wrapper: createTestWrapper() });

      // Then: The primary loop should not have a merge button
      const activeLoopsContainer = screen
        .getByRole("heading", { name: /active loops/i })
        .closest(".rounded-lg") as HTMLElement;

      // Find the loop row for primary loop
      const primaryLoopRow = within(activeLoopsContainer)
        .getByText(/building core module/i)
        .closest(".bg-muted\\/50") as HTMLElement;

      // Primary loops are in-place and should have Stop button but not Merge
      expect(within(primaryLoopRow).queryByRole("button", { name: /merge now/i })).not.toBeInTheDocument();
    });
  });
});
