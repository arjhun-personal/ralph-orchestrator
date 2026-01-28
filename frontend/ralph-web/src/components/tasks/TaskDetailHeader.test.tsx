/**
 * TaskDetailHeader Component Tests
 *
 * Tests for the TaskDetailHeader component that displays:
 * - Left side: Back navigation button ("← Back to Tasks")
 * - Right side: Status-based action button (Cancel/Retry/Run)
 */

import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent } from "@testing-library/react";
import { TaskDetailHeader, type TaskDetailHeaderProps } from "./TaskDetailHeader";

describe("TaskDetailHeader", () => {
  const defaultProps: TaskDetailHeaderProps = {
    status: "open",
    onBack: vi.fn(),
    onAction: vi.fn(),
  };

  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe("back navigation", () => {
    it("renders back button with arrow icon and text", () => {
      render(<TaskDetailHeader {...defaultProps} />);

      // Should have a back button
      const backButton = screen.getByRole("button", { name: /back to tasks/i });
      expect(backButton).toBeInTheDocument();

      // Should have arrow icon (ArrowLeft from lucide-react)
      expect(document.querySelector(".lucide-arrow-left")).toBeInTheDocument();
    });

    it("calls onBack when back button is clicked", () => {
      const onBack = vi.fn();
      render(<TaskDetailHeader {...defaultProps} onBack={onBack} />);

      const backButton = screen.getByRole("button", { name: /back to tasks/i });
      fireEvent.click(backButton);

      expect(onBack).toHaveBeenCalledTimes(1);
    });

    it("back button has text variant styling", () => {
      render(<TaskDetailHeader {...defaultProps} />);

      const backButton = screen.getByRole("button", { name: /back to tasks/i });
      // Text buttons typically have "ghost" or similar variant
      expect(backButton).toHaveClass("gap-1"); // Icon and text should have gap
    });
  });

  describe("status-based action buttons", () => {
    describe("when status is 'running'", () => {
      it("renders Cancel button with destructive variant", () => {
        render(<TaskDetailHeader {...defaultProps} status="running" />);

        const cancelButton = screen.getByRole("button", { name: /cancel/i });
        expect(cancelButton).toBeInTheDocument();
        // Destructive variant typically has red styling
        expect(cancelButton).toHaveClass("bg-destructive");
      });

      it("calls onAction with 'cancel' when Cancel is clicked", () => {
        const onAction = vi.fn();
        render(
          <TaskDetailHeader {...defaultProps} status="running" onAction={onAction} />
        );

        const cancelButton = screen.getByRole("button", { name: /cancel/i });
        fireEvent.click(cancelButton);

        expect(onAction).toHaveBeenCalledWith("cancel");
      });
    });

    describe("when status is 'failed'", () => {
      it("renders Retry button", () => {
        render(<TaskDetailHeader {...defaultProps} status="failed" />);

        const retryButton = screen.getByRole("button", { name: /retry/i });
        expect(retryButton).toBeInTheDocument();
      });

      it("calls onAction with 'retry' when Retry is clicked", () => {
        const onAction = vi.fn();
        render(
          <TaskDetailHeader {...defaultProps} status="failed" onAction={onAction} />
        );

        const retryButton = screen.getByRole("button", { name: /retry/i });
        fireEvent.click(retryButton);

        expect(onAction).toHaveBeenCalledWith("retry");
      });
    });

    describe("when status is 'open'", () => {
      it("renders Run button with primary variant", () => {
        render(<TaskDetailHeader {...defaultProps} status="open" />);

        const runButton = screen.getByRole("button", { name: /run/i });
        expect(runButton).toBeInTheDocument();
        // Primary variant typically has primary color
        expect(runButton).toHaveClass("bg-primary");
      });

      it("calls onAction with 'run' when Run is clicked", () => {
        const onAction = vi.fn();
        render(
          <TaskDetailHeader {...defaultProps} status="open" onAction={onAction} />
        );

        const runButton = screen.getByRole("button", { name: /run/i });
        fireEvent.click(runButton);

        expect(onAction).toHaveBeenCalledWith("run");
      });
    });

    describe("when status is 'completed'", () => {
      it("does not render any action button", () => {
        render(<TaskDetailHeader {...defaultProps} status="completed" />);

        // Should have back button but no action buttons
        expect(screen.getByRole("button", { name: /back to tasks/i })).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: /run/i })).not.toBeInTheDocument();
        expect(screen.queryByRole("button", { name: /cancel/i })).not.toBeInTheDocument();
        expect(screen.queryByRole("button", { name: /retry/i })).not.toBeInTheDocument();
      });
    });

    describe("when status is 'closed'", () => {
      it("does not render any action button", () => {
        render(<TaskDetailHeader {...defaultProps} status="closed" />);

        // Should have back button but no action buttons
        expect(screen.getByRole("button", { name: /back to tasks/i })).toBeInTheDocument();
        expect(screen.queryByRole("button", { name: /run/i })).not.toBeInTheDocument();
        expect(screen.queryByRole("button", { name: /cancel/i })).not.toBeInTheDocument();
        expect(screen.queryByRole("button", { name: /retry/i })).not.toBeInTheDocument();
      });
    });
  });

  describe("layout", () => {
    it("positions back button on the left and action button on the right", () => {
      render(<TaskDetailHeader {...defaultProps} status="open" />);

      // Container should use flexbox with space-between
      const container = screen.getByRole("button", { name: /back to tasks/i }).parentElement;
      expect(container).toHaveClass("flex");
      expect(container).toHaveClass("justify-between");
    });

    it("aligns items vertically in the center", () => {
      render(<TaskDetailHeader {...defaultProps} status="open" />);

      const container = screen.getByRole("button", { name: /back to tasks/i }).parentElement;
      expect(container).toHaveClass("items-center");
    });
  });

  describe("when onAction is not provided", () => {
    it("action buttons are disabled when onAction is undefined", () => {
      render(<TaskDetailHeader status="open" onBack={vi.fn()} />);

      const runButton = screen.getByRole("button", { name: /run/i });
      expect(runButton).toBeDisabled();
    });
  });

  describe("loading states", () => {
    it("disables action button when isActionPending is true", () => {
      render(
        <TaskDetailHeader
          {...defaultProps}
          status="open"
          isActionPending={true}
        />
      );

      const runButton = screen.getByRole("button", { name: /run/i });
      expect(runButton).toBeDisabled();
    });

    it("shows loading indicator when isActionPending is true", () => {
      render(
        <TaskDetailHeader
          {...defaultProps}
          status="running"
          isActionPending={true}
        />
      );

      // Loader2 icon from lucide-react with animate-spin
      expect(document.querySelector(".lucide-loader-2")).toBeInTheDocument();
      expect(document.querySelector(".animate-spin")).toBeInTheDocument();
    });
  });
});
