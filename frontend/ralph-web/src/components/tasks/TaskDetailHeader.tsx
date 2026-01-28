/**
 * TaskDetailHeader Component
 *
 * Header for task detail view with navigation and status-based actions.
 * - Left side: Back navigation button ("← Back to Tasks")
 * - Right side: Status-based action button (Cancel/Retry/Run)
 */

import { Button } from "@/components/ui/button";
import { ArrowLeft, Loader2 } from "lucide-react";

export type TaskStatus = "open" | "running" | "completed" | "closed" | "failed";
export type TaskAction = "run" | "cancel" | "retry";

export interface TaskDetailHeaderProps {
  /** Current task status */
  status: TaskStatus;
  /** Callback when back button is clicked */
  onBack: () => void;
  /** Callback when action button is clicked - if undefined, action button is disabled */
  onAction?: (action: TaskAction) => void;
  /** Whether an action is pending (shows loading state) */
  isActionPending?: boolean;
}

/**
 * Get the action configuration for a given status
 */
function getActionForStatus(status: TaskStatus): { action: TaskAction; label: string; variant: "default" | "destructive" } | null {
  switch (status) {
    case "running":
      return { action: "cancel", label: "Cancel", variant: "destructive" };
    case "failed":
      return { action: "retry", label: "Retry", variant: "default" };
    case "open":
      return { action: "run", label: "Run", variant: "default" };
    case "completed":
    case "closed":
      return null;
    default:
      return null;
  }
}

export function TaskDetailHeader({
  status,
  onBack,
  onAction,
  isActionPending = false,
}: TaskDetailHeaderProps) {
  const actionConfig = getActionForStatus(status);

  return (
    <div className="flex justify-between items-center">
      <Button
        variant="ghost"
        className="gap-1"
        onClick={onBack}
      >
        <ArrowLeft className="lucide-arrow-left" />
        Back to Tasks
      </Button>

      {actionConfig && (
        <Button
          variant={actionConfig.variant}
          onClick={() => onAction?.(actionConfig.action)}
          disabled={!onAction || isActionPending}
        >
          {isActionPending && <Loader2 className="lucide-loader-2 animate-spin" />}
          {actionConfig.label}
        </Button>
      )}
    </div>
  );
}
