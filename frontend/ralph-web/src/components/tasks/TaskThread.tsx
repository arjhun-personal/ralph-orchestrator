/**
 * TaskThread Component
 *
 * A collapsible thread display for a single task. Shows task title,
 * status badge, and timestamp in collapsed state. Expands to show
 * full details with real-time log streaming via EnhancedLogViewer.
 *
 * For running tasks, displays a LiveStatus component with real-time
 * WebSocket updates showing the latest status line.
 */

import { useMemo, useCallback, forwardRef, type MouseEvent, memo, useState } from "react";
import {
  CheckCircle2,
  Circle,
  Clock,
  Loader2,
  XCircle,
  ChevronRight,
  ChevronDown,
  Play,
  RotateCcw,
  Archive,
  Square,
} from "lucide-react";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { useUIStore } from "@/store";
import { LiveStatus } from "./LiveStatus";
import { EnhancedLogViewer } from "./EnhancedLogViewer";
import { trpc } from "@/trpc";
import { LoopBadge } from "./LoopBadge";
import { LoopDetail, type LoopDetailData } from "./LoopDetail";
import { LoopActions, type LoopActionCallbacks } from "./LoopActions";

/**
 * Task shape from the tRPC API.
 * Note: Dates come as ISO strings over JSON, so we accept both Date and string.
 */
export interface Task {
  id: string;
  title: string;
  status: string;
  priority: number;
  blockedBy: string | null;
  createdAt: Date | string;
  updatedAt: Date | string;
  // Execution tracking fields
  queuedTaskId?: string | null;
  startedAt?: Date | string | null;
  completedAt?: Date | string | null;
  errorMessage?: string | null;
  // Execution summary fields
  executionSummary?: string | null;
  exitCode?: number | null;
  durationMs?: number | null;
  archivedAt?: Date | string | null;
  // PID field for task↔loop mapping per spec lines 65-68
  // Backend must populate this from ProcessSupervisor for running tasks
  pid?: number | null;
}

interface TaskThreadProps {
  /** The task to display */
  task: Task;
  /** Optional loop data for loop visibility per spec lines 100-117 */
  loop?: LoopDetailData;
  /** Whether this task is focused via keyboard navigation */
  isFocused?: boolean;
  /** Additional CSS classes */
  className?: string;
}

/**
 * Status configuration for visual styling
 */
interface StatusConfig {
  icon: typeof Circle;
  color: string;
  badgeVariant: "default" | "secondary" | "destructive" | "outline";
  label: string;
}

const STATUS_MAP: Record<string, StatusConfig> = {
  open: {
    icon: Circle,
    color: "text-zinc-400",
    badgeVariant: "secondary",
    label: "Open",
  },
  pending: {
    icon: Clock,
    color: "text-yellow-500",
    badgeVariant: "outline",
    label: "Pending",
  },
  running: {
    icon: Loader2,
    color: "text-blue-500",
    badgeVariant: "default",
    label: "Running",
  },
  completed: {
    icon: CheckCircle2,
    color: "text-green-500",
    badgeVariant: "secondary",
    label: "Completed",
  },
  closed: {
    icon: CheckCircle2,
    color: "text-green-500",
    badgeVariant: "secondary",
    label: "Closed",
  },
  failed: {
    icon: XCircle,
    color: "text-red-500",
    badgeVariant: "destructive",
    label: "Failed",
  },
  cancelled: {
    icon: XCircle,
    color: "text-orange-500",
    badgeVariant: "outline",
    label: "Cancelled",
  },
  archived: {
    icon: Archive,
    color: "text-zinc-500",
    badgeVariant: "outline",
    label: "Archived",
  },
  blocked: {
    icon: Clock,
    color: "text-orange-500",
    badgeVariant: "outline",
    label: "Blocked",
  },
};

const DEFAULT_STATUS: StatusConfig = {
  icon: Circle,
  color: "text-zinc-400",
  badgeVariant: "outline",
  label: "Unknown",
};

/**
 * Format a relative time string (e.g., "2 hours ago", "just now")
 */
function formatRelativeTime(date: Date): string {
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSecs < 60) return "just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;

  return date.toLocaleDateString();
}

/**
 * Format a duration in milliseconds to a human-readable string
 */
function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const secs = Math.floor(ms / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  const remainingSecs = secs % 60;
  if (mins < 60) return remainingSecs > 0 ? `${mins}m ${remainingSecs}s` : `${mins}m`;
  const hours = Math.floor(mins / 60);
  const remainingMins = mins % 60;
  return remainingMins > 0 ? `${hours}h ${remainingMins}m` : `${hours}h`;
}

/**
 * Simple markdown section parser - extracts content between headers
 */
function parseMarkdownSection(markdown: string, sectionName: string): string | null {
  const regex = new RegExp(`## ${sectionName}\\s*\\n([\\s\\S]*?)(?=\\n## |$)`, "i");
  const match = markdown.match(regex);
  return match ? match[1].trim() : null;
}

const TaskThreadComponent = forwardRef<HTMLDivElement, TaskThreadProps>(function TaskThread(
  { task, loop, isFocused = false, className },
  ref
) {
  const isExpanded = useUIStore((state) => state.expandedTasks.has(task.id));
  const toggleTaskExpanded = useUIStore((state) => state.toggleTaskExpanded);

  const statusConfig = useMemo(() => {
    if (task.archivedAt) return STATUS_MAP.archived;
    return STATUS_MAP[task.status] || DEFAULT_STATUS;
  }, [task.status, task.archivedAt]);

  const StatusIcon = statusConfig.icon;
  const isArchived = !!task.archivedAt;
  const isArchivedFailed = isArchived && (!!task.errorMessage || (task.exitCode ?? 0) !== 0);
  const isRunning = task.status === "running";
  const isFailed = task.status === "failed" || isArchivedFailed;
  const isCompleted =
    task.status === "completed" || task.status === "closed" || (isArchived && !isArchivedFailed);
  const isOpen = task.status === "open";

  // Show log viewer for running tasks or terminal tasks (logs replay on subscribe)
  const shouldShowLogViewer = isRunning || isCompleted || isFailed;

  // Can run: open or pending (not yet running)
  const canRun = isOpen && !task.blockedBy;
  // Can retry: only failed tasks
  const canRetry = isFailed;

  // tRPC mutations
  const utils = trpc.useUtils();
  const runMutation = trpc.task.run.useMutation({
    onSuccess: () => {
      utils.task.list.invalidate();
    },
  });
  const retryMutation = trpc.task.retry.useMutation({
    onSuccess: () => {
      utils.task.list.invalidate();
    },
  });
  const cancelMutation = trpc.task.cancel.useMutation({
    onSuccess: () => {
      utils.task.list.invalidate();
    },
  });

  // Loop mutations for merge queue actions (spec lines 106-110)
  const loopRetryMutation = trpc.loops.retry.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });
  const loopMergeMutation = trpc.loops.merge.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });
  const loopDiscardMutation = trpc.loops.discard.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });
  const loopStopMutation = trpc.loops.stop.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });

  // Create action callbacks for loop actions
  const loopActionCallbacks: LoopActionCallbacks | undefined = useMemo(() => {
    if (!loop) return undefined;
    return {
      onRetry: async () => {
        await loopRetryMutation.mutateAsync({ id: loop.id });
      },
      onMerge: async () => {
        await loopMergeMutation.mutateAsync({ id: loop.id });
      },
      onDiscard: async () => {
        await loopDiscardMutation.mutateAsync({ id: loop.id });
      },
      onStop: async () => {
        await loopStopMutation.mutateAsync({ id: loop.id });
      },
    };
  }, [loop, loopRetryMutation, loopMergeMutation, loopDiscardMutation, loopStopMutation]);

  // Local state for cancel confirmation
  const [showCancelConfirm, setShowCancelConfirm] = useState(false);

  const handleRun = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      runMutation.mutate({ id: task.id });
    },
    [task.id, runMutation]
  );

  const handleRetry = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      retryMutation.mutate({ id: task.id });
    },
    [task.id, retryMutation]
  );

  const handleCancelClick = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      setShowCancelConfirm(true);
    },
    []
  );

  const handleCancelConfirm = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      cancelMutation.mutate({ id: task.id });
      setShowCancelConfirm(false);
    },
    [task.id, cancelMutation]
  );

  const handleCancelCancel = useCallback(
    (e: MouseEvent) => {
      e.stopPropagation();
      setShowCancelConfirm(false);
    },
    []
  );

  const handleToggle = useCallback(
    (e: MouseEvent) => {
      e.preventDefault();
      toggleTaskExpanded(task.id);
    },
    [task.id, toggleTaskExpanded]
  );

  const relativeTime = useMemo(
    () => formatRelativeTime(new Date(task.updatedAt)),
    [task.updatedAt]
  );

  const summarySections = useMemo(() => {
    if (!isExpanded || !task.executionSummary) return null;
    return {
      whatWasDone: parseMarkdownSection(task.executionSummary, "What Was Done"),
      keyChanges: parseMarkdownSection(task.executionSummary, "Key Changes"),
      notes: parseMarkdownSection(task.executionSummary, "Notes"),
      tasks: parseMarkdownSection(task.executionSummary, "Tasks"),
      finalCommit: parseMarkdownSection(task.executionSummary, "Final Commit"),
    };
  }, [isExpanded, task.executionSummary]);

  const isExecuting = runMutation.isPending || retryMutation.isPending || cancelMutation.isPending;

  return (
    <Card
      ref={ref}
      className={cn(
        "transition-all duration-200 cursor-pointer",
        !isExpanded && "hover:bg-accent/50",
        isExpanded && "ring-1 ring-accent",
        isFocused && "ring-2 ring-primary bg-accent/30",
        className
      )}
      onClick={handleToggle}
      role="button"
      tabIndex={0}
      aria-expanded={isExpanded}
      onKeyDown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.preventDefault();
          toggleTaskExpanded(task.id);
        }
      }}
    >
      <CardHeader className="p-4">
        <div className="flex flex-col gap-2">
          <div className="flex items-center gap-3">
            {/* Expand/collapse chevron */}
            <span className="text-muted-foreground shrink-0">
              {isExpanded ? (
                <ChevronDown className="h-4 w-4" />
              ) : (
                <ChevronRight className="h-4 w-4" />
              )}
            </span>

            {/* Status icon */}
            <StatusIcon
              className={cn("h-5 w-5 shrink-0", statusConfig.color, isRunning && "animate-spin")}
              aria-hidden="true"
            />

            {/* Task title */}
            <span className={cn("font-medium flex-1", !isExpanded && "truncate")}>
              {task.title}
            </span>

            {/* Status badge */}
            <Badge variant={statusConfig.badgeVariant} className="shrink-0">
              {statusConfig.label}
            </Badge>

            {/* Loop badge - only shown when a loop match exists (spec line 150) */}
            {loop && <LoopBadge status={loop.status} className="shrink-0" />}

            {/* Run/Retry/Cancel buttons */}
            {canRun && (
              <Button
                size="sm"
                variant="ghost"
                className="shrink-0 h-7 px-2"
                onClick={handleRun}
                disabled={isExecuting}
              >
                {isExecuting ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Play className="h-4 w-4" />
                )}
                <span className="ml-1">Run</span>
              </Button>
            )}
            {canRetry && (
              <Button
                size="sm"
                variant="ghost"
                className="shrink-0 h-7 px-2"
                onClick={handleRetry}
                disabled={isExecuting}
              >
                {isExecuting ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <RotateCcw className="h-4 w-4" />
                )}
                <span className="ml-1">Retry</span>
              </Button>
            )}
            {/* Cancel button for running tasks */}
            {isRunning && !showCancelConfirm && (
              <Button
                size="sm"
                variant="ghost"
                className="shrink-0 h-7 px-2 text-orange-500 hover:text-orange-600 hover:bg-orange-50 dark:hover:bg-orange-950/20"
                onClick={handleCancelClick}
                disabled={isExecuting}
              >
                {isExecuting ? (
                  <Loader2 className="h-4 w-4 animate-spin" />
                ) : (
                  <Square className="h-4 w-4" />
                )}
                <span className="ml-1">Cancel</span>
              </Button>
            )}
            {/* Cancel confirmation for running tasks */}
            {isRunning && showCancelConfirm && (
              <div className="flex items-center gap-1 shrink-0">
                <span className="text-xs text-muted-foreground">Stop task?</span>
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-7 px-2 text-red-500 hover:text-red-600 hover:bg-red-50 dark:hover:bg-red-950/20"
                  onClick={handleCancelConfirm}
                  disabled={isExecuting}
                >
                  {isExecuting ? (
                    <Loader2 className="h-4 w-4 animate-spin" />
                  ) : (
                    <Square className="h-4 w-4" />
                  )}
                  <span className="ml-1">Yes</span>
                </Button>
                <Button
                  size="sm"
                  variant="ghost"
                  className="h-7 px-2"
                  onClick={handleCancelCancel}
                  disabled={isExecuting}
                >
                  No
                </Button>
              </div>
            )}

            {/* Relative time */}
            <span className="text-xs text-muted-foreground shrink-0 tabular-nums">
              {relativeTime}
            </span>
          </div>

          {/* Live status for running tasks (only in collapsed view) */}
          {isRunning && !isExpanded && <LiveStatus taskId={task.id} className="ml-12" />}
        </div>
      </CardHeader>

      {/* Expanded content area */}
      {isExpanded && (
        <CardContent className="pt-0 pb-4">
          <div className="ml-12 border-l-2 border-border pl-4" onClick={(e) => e.stopPropagation()}>
            {/* Task metadata */}
            <div className="flex items-center gap-4 mb-3 text-xs text-muted-foreground">
              <span>Created: {new Date(task.createdAt).toLocaleString()}</span>
              {task.blockedBy && (
                <span className="text-orange-400">Blocked by: {task.blockedBy}</span>
              )}
            </div>

            {/* Loop details - only shown when a loop match exists (spec lines 115-117, 151-152) */}
            {loop && (
              <div className="space-y-3 mb-3">
                <LoopDetail loop={loop} defaultExpanded={false} />
                <LoopActions
                  id={loop.id}
                  status={loop.status}
                  isGitWorkspace={loop.repoRoot !== null && loop.repoRoot !== undefined}
                  callbacks={loopActionCallbacks}
                />
              </div>
            )}

            {/* Execution summary for completed/failed tasks (shown above logs) */}
            {!isRunning && (isCompleted || isFailed) && (
              <div className="bg-zinc-900/50 rounded-md p-4 text-sm text-zinc-400 mb-3">
                {isCompleted ? (
                  <div className="space-y-4">
                    {/* Duration badge */}
                    {task.durationMs && (
                      <div className="flex items-center gap-2 text-xs text-zinc-500">
                        <Clock className="h-3 w-3" />
                        <span>Completed in {formatDuration(task.durationMs)}</span>
                      </div>
                    )}

                    {/* Execution summary */}
                    {task.executionSummary ? (
                      <div className="space-y-3">
                        {/* Try new summarizer format first */}
                        {summarySections?.whatWasDone ? (
                          <>
                            <div>
                              <h4 className="text-green-400 font-medium mb-1">What Was Done</h4>
                              <p className="text-zinc-300 text-sm leading-relaxed">
                                {summarySections.whatWasDone}
                              </p>
                            </div>
                            {summarySections.keyChanges && (
                              <div>
                                <h4 className="text-blue-400 font-medium mb-1">Key Changes</h4>
                                <pre className="text-zinc-300 text-xs whitespace-pre-wrap font-mono">
                                  {summarySections.keyChanges}
                                </pre>
                              </div>
                            )}
                            {summarySections.notes && (
                              <div>
                                <h4 className="text-yellow-400 font-medium mb-1">Notes</h4>
                                <p className="text-zinc-400 text-xs">{summarySections.notes}</p>
                              </div>
                            )}
                          </>
                        ) : summarySections?.tasks ? (
                          /* Fallback to ralph's native Loop Summary format */
                          <>
                            <div>
                              <h4 className="text-green-400 font-medium mb-1">Tasks Completed</h4>
                              <pre className="text-zinc-300 text-xs whitespace-pre-wrap font-mono">
                                {summarySections.tasks}
                              </pre>
                            </div>
                            {summarySections.finalCommit && (
                              <div>
                                <h4 className="text-blue-400 font-medium mb-1">Final Commit</h4>
                                <p className="text-zinc-300 text-sm font-mono">
                                  {summarySections.finalCommit}
                                </p>
                              </div>
                            )}
                          </>
                        ) : (
                          /* Generic fallback - show raw summary */
                          <div>
                            <h4 className="text-green-400 font-medium mb-1">Execution Summary</h4>
                            <pre className="text-zinc-300 text-xs whitespace-pre-wrap font-mono max-h-48 overflow-y-auto">
                              {task.executionSummary.slice(0, 1000)}
                              {task.executionSummary.length > 1000 && "..."}
                            </pre>
                          </div>
                        )}
                      </div>
                    ) : (
                      <p className="text-green-400">Task completed successfully.</p>
                    )}
                  </div>
                ) : (
                  /* Failed task */
                  <div className="space-y-2">
                    {task.durationMs && (
                      <div className="flex items-center gap-2 text-xs text-zinc-500">
                        <Clock className="h-3 w-3" />
                        <span>Failed after {formatDuration(task.durationMs)}</span>
                      </div>
                    )}
                    <p className="text-red-400">Task failed.</p>
                    {task.errorMessage && (
                      <p className="text-red-300 text-xs">Error: {task.errorMessage}</p>
                    )}
                  </div>
                )}
              </div>
            )}

            {/* Log viewer for running tasks OR completed/failed tasks with logs */}
            {shouldShowLogViewer && <EnhancedLogViewer taskId={task.id} height="300px" />}

            {/* Static info for other non-running tasks (open, pending, blocked) */}
            {!isRunning && !isCompleted && !isFailed && (
              <div className="bg-zinc-900/50 rounded-md p-4 text-sm text-zinc-400">
                {task.status === "pending" ? (
                  <p>Task is queued and waiting to execute...</p>
                ) : (
                  <p>Task is waiting to run.</p>
                )}
              </div>
            )}
          </div>
        </CardContent>
      )}
    </Card>
  );
});

TaskThreadComponent.displayName = "TaskThread";

function getUpdatedAtValue(value: Date | string): string {
  return typeof value === "string" ? value : value.toISOString();
}

const areTasksEqual = (prev: TaskThreadProps, next: TaskThreadProps): boolean => {
  if (prev.isFocused !== next.isFocused) return false;
  if (prev.className !== next.className) return false;
  if (prev.task.id !== next.task.id) return false;
  if (prev.task.status !== next.task.status) return false;
  if (prev.task.title !== next.task.title) return false;
  if (prev.task.blockedBy !== next.task.blockedBy) return false;
  if (getUpdatedAtValue(prev.task.updatedAt) !== getUpdatedAtValue(next.task.updatedAt)) {
    return false;
  }
  const prevArchived = prev.task.archivedAt ? getUpdatedAtValue(prev.task.archivedAt) : null;
  const nextArchived = next.task.archivedAt ? getUpdatedAtValue(next.task.archivedAt) : null;
  if (prevArchived !== nextArchived) return false;

  // Compare loop props for re-render when loop state changes
  if (prev.loop?.id !== next.loop?.id) return false;
  if (prev.loop?.status !== next.loop?.status) return false;

  return true;
};

export const TaskThread = memo(TaskThreadComponent, areTasksEqual);
