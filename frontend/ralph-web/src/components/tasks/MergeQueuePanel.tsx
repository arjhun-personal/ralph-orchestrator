/**
 * MergeQueuePanel Component
 *
 * A compact panel for managing the merge queue on the Tasks page.
 * Shows non-task loops (orphans, manual merges, loops without PID mapping)
 * with summary counts, primary lock info, and global actions.
 *
 * Per spec lines 119-127, 159-162:
 * - Panel lists loops not mapped to any task (orphans/manual merges)
 * - Panel has "Process queue" and "Prune stale" actions
 * - Panel supports "Show merged/discarded" toggle (default OFF)
 */

import { memo, useState, useCallback, useMemo } from "react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { trpc } from "@/trpc";
import { LoopBadge, type LoopStatus } from "./LoopBadge";
import { LoopDetail, type LoopDetailData } from "./LoopDetail";
import { LoopActions, type LoopActionCallbacks } from "./LoopActions";
import {
  RefreshCw,
  Trash2,
  Play,
  ChevronDown,
  ChevronRight,
  AlertTriangle,
  Loader2,
  Lock,
  Clock,
} from "lucide-react";

interface MergeQueuePanelProps {
  /** Polling interval in milliseconds. Set to 0 to disable. Default: 5000 */
  pollingInterval?: number;
  /** Additional CSS classes */
  className?: string;
  /** Initial expanded state for the panel */
  defaultExpanded?: boolean;
}

/**
 * Format a relative time string (e.g., "2m ago", "1h ago")
 */
function formatRelativeAge(isoDate: string): string {
  const date = new Date(isoDate);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSecs = Math.floor(diffMs / 1000);
  const diffMins = Math.floor(diffSecs / 60);
  const diffHours = Math.floor(diffMins / 60);
  const diffDays = Math.floor(diffHours / 24);

  if (diffSecs < 60) return `${diffSecs}s ago`;
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}

/**
 * Summary of loop states for the header
 */
interface LoopSummary {
  running: number;
  queued: number;
  merging: number;
  needsReview: number;
  crashed: number;
  orphan: number;
  merged: number;
  discarded: number;
  total: number;
  totalActive: number; // excludes terminal states
}

function computeLoopSummary(loops: LoopDetailData[]): LoopSummary {
  const summary: LoopSummary = {
    running: 0,
    queued: 0,
    merging: 0,
    needsReview: 0,
    crashed: 0,
    orphan: 0,
    merged: 0,
    discarded: 0,
    total: 0,
    totalActive: 0,
  };

  for (const loop of loops) {
    summary.total++;
    switch (loop.status) {
      case "running":
        summary.running++;
        summary.totalActive++;
        break;
      case "queued":
        summary.queued++;
        summary.totalActive++;
        break;
      case "merging":
        summary.merging++;
        summary.totalActive++;
        break;
      case "needs-review":
        summary.needsReview++;
        summary.totalActive++;
        break;
      case "crashed":
        summary.crashed++;
        summary.totalActive++;
        break;
      case "orphan":
        summary.orphan++;
        summary.totalActive++;
        break;
      case "merged":
        summary.merged++;
        break;
      case "discarded":
        summary.discarded++;
        break;
    }
  }

  return summary;
}

function MergeQueuePanelComponent({
  pollingInterval = 5000,
  className,
  defaultExpanded = true,
}: MergeQueuePanelProps) {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);
  const [showTerminal, setShowTerminal] = useState(false);
  const [processingAction, setProcessingAction] = useState<string | null>(null);
  const [actionError, setActionError] = useState<string | null>(null);

  // Fetch loops with optional terminal states
  const loopsQuery = trpc.loops.list.useQuery(
    { includeTerminal: showTerminal },
    {
      refetchInterval: pollingInterval > 0 ? pollingInterval : false,
    }
  );

  // Manager status for primary lock info
  const managerStatusQuery = trpc.loops.managerStatus.useQuery(undefined, {
    refetchInterval: pollingInterval > 0 ? pollingInterval : false,
  });

  // Mutations for global actions
  const utils = trpc.useUtils();
  const processMutation = trpc.loops.process.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });
  const pruneMutation = trpc.loops.prune.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });
  const retryMutation = trpc.loops.retry.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });
  const discardMutation = trpc.loops.discard.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });
  const stopMutation = trpc.loops.stop.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });
  const mergeMutation = trpc.loops.merge.useMutation({
    onSuccess: () => utils.loops.list.invalidate(),
  });

  // Convert API response to LoopDetailData
  const loops: LoopDetailData[] = useMemo(() => {
    if (!loopsQuery.data) return [];
    return (loopsQuery.data as LoopDetailData[]).map((loop) => ({
      ...loop,
      status: loop.status as LoopStatus,
    }));
  }, [loopsQuery.data]);

  // Compute summary for header
  const summary = useMemo(() => computeLoopSummary(loops), [loops]);

  // Find primary loop for lock info
  const primaryLoop = useMemo(
    () => loops.find((loop) => loop.isPrimary),
    [loops]
  );

  // Handle panel toggle
  const handleToggle = useCallback(() => {
    setIsExpanded((prev) => !prev);
  }, []);

  // Handle terminal toggle
  const handleShowTerminalToggle = useCallback(() => {
    setShowTerminal((prev) => !prev);
  }, []);

  // Handle process queue action
  const handleProcessQueue = useCallback(async () => {
    setActionError(null);
    setProcessingAction("process");
    try {
      await processMutation.mutateAsync();
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to process queue";
      setActionError(message);
    } finally {
      setProcessingAction(null);
    }
  }, [processMutation]);

  // Handle prune stale action
  const handlePruneStale = useCallback(async () => {
    setActionError(null);
    setProcessingAction("prune");
    try {
      await pruneMutation.mutateAsync();
    } catch (err) {
      const message = err instanceof Error ? err.message : "Failed to prune stale loops";
      setActionError(message);
    } finally {
      setProcessingAction(null);
    }
  }, [pruneMutation]);

  // Create action callbacks for individual loops
  const createActionCallbacks = useCallback(
    (loopId: string): LoopActionCallbacks => ({
      onRetry: async () => {
        await retryMutation.mutateAsync({ id: loopId });
      },
      onMerge: async () => {
        await mergeMutation.mutateAsync({ id: loopId });
      },
      onDiscard: async () => {
        await discardMutation.mutateAsync({ id: loopId });
      },
      onStop: async () => {
        await stopMutation.mutateAsync({ id: loopId });
      },
    }),
    [retryMutation, mergeMutation, discardMutation, stopMutation]
  );

  // Error state from loopsQuery
  if (loopsQuery.isError) {
    return (
      <div className={cn("rounded-lg border border-destructive/50 p-4", className)}>
        <div className="flex items-center gap-2 text-sm text-destructive">
          <AlertTriangle className="h-4 w-4 shrink-0" />
          <span>Failed to load merge queue: {loopsQuery.error.message}</span>
        </div>
        <Button
          variant="outline"
          size="sm"
          onClick={() => loopsQuery.refetch()}
          disabled={loopsQuery.isFetching}
          className="mt-2"
        >
          <RefreshCw className={cn("h-4 w-4 mr-2", loopsQuery.isFetching && "animate-spin")} />
          Retry
        </Button>
      </div>
    );
  }

  // Build summary string
  const summaryParts: string[] = [];
  if (summary.running > 0) summaryParts.push(`${summary.running} running`);
  if (summary.queued > 0) summaryParts.push(`${summary.queued} queued`);
  if (summary.merging > 0) summaryParts.push(`${summary.merging} merging`);
  if (summary.needsReview > 0) summaryParts.push(`${summary.needsReview} needs-review`);
  if (summary.crashed > 0) summaryParts.push(`${summary.crashed} crashed`);
  if (summary.orphan > 0) summaryParts.push(`${summary.orphan} orphan`);
  if (showTerminal) {
    if (summary.merged > 0) summaryParts.push(`${summary.merged} merged`);
    if (summary.discarded > 0) summaryParts.push(`${summary.discarded} discarded`);
  }

  const summaryText = summaryParts.length > 0 ? summaryParts.join(" | ") : "No active loops";

  return (
    <div className={cn("border border-border rounded-lg bg-card", className)}>
      {/* Header - always visible */}
      <button
        type="button"
        className="w-full flex items-center gap-2 px-4 py-3 text-sm hover:bg-accent/50 transition-colors rounded-t-lg"
        onClick={handleToggle}
        aria-expanded={isExpanded}
      >
        {isExpanded ? (
          <ChevronDown className="h-4 w-4 text-muted-foreground shrink-0" />
        ) : (
          <ChevronRight className="h-4 w-4 text-muted-foreground shrink-0" />
        )}
        <span className="font-medium">Merge Queue</span>
        <span className="text-muted-foreground text-xs ml-2">{summaryText}</span>
        
        {/* Primary lock info */}
        {primaryLoop && (
          <span className="ml-auto text-xs text-muted-foreground flex items-center gap-1">
            <Lock className="h-3 w-3" />
            PID {primaryLoop.pid}
            {primaryLoop.startedAt && (
              <span className="flex items-center gap-1">
                <Clock className="h-3 w-3" />
                {formatRelativeAge(primaryLoop.startedAt)}
              </span>
            )}
          </span>
        )}
        
        {/* Loading/refresh indicator */}
        {loopsQuery.isFetching && (
          <RefreshCw className="h-3 w-3 animate-spin text-muted-foreground ml-2" />
        )}
      </button>

      {/* Expanded content */}
      {isExpanded && (
        <div className="px-4 py-3 border-t border-border space-y-4">
          {/* Global actions row */}
          <div className="flex flex-wrap items-center gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleProcessQueue}
              disabled={processingAction !== null}
              title="Process pending merges in the queue"
            >
              {processingAction === "process" ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : (
                <Play className="h-3 w-3" />
              )}
              <span className="ml-1">Process Queue</span>
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handlePruneStale}
              disabled={processingAction !== null}
              title="Remove stale loops from crashed processes"
            >
              {processingAction === "prune" ? (
                <Loader2 className="h-3 w-3 animate-spin" />
              ) : (
                <Trash2 className="h-3 w-3" />
              )}
              <span className="ml-1">Prune Stale</span>
            </Button>
            
            <div className="flex-1" />
            
            {/* Toggle for terminal states */}
            <Button
              variant={showTerminal ? "secondary" : "ghost"}
              size="sm"
              onClick={handleShowTerminalToggle}
              title={showTerminal ? "Hide merged/discarded loops" : "Show merged/discarded loops"}
            >
              {showTerminal ? "Hide merged/discarded" : "Show merged/discarded"}
            </Button>
            
            {/* Manual refresh */}
            <Button
              variant="ghost"
              size="sm"
              onClick={() => loopsQuery.refetch()}
              disabled={loopsQuery.isFetching}
              title="Refresh loop list"
              className="h-7 px-2"
            >
              <RefreshCw className={cn("h-3.5 w-3.5", loopsQuery.isFetching && "animate-spin")} />
              <span className="sr-only">Refresh</span>
            </Button>
          </div>

          {/* Error message */}
          {actionError && (
            <div className="flex items-center gap-2 text-sm text-destructive">
              <AlertTriangle className="h-4 w-4 shrink-0" />
              <span>{actionError}</span>
            </div>
          )}

          {/* Manager status */}
          {managerStatusQuery.data && (
            <div className="text-xs text-muted-foreground">
              Manager: {managerStatusQuery.data.running ? "running" : "stopped"}
              {managerStatusQuery.data.running && (
                <span> (every {Math.round(managerStatusQuery.data.intervalMs / 1000)}s)</span>
              )}
              {managerStatusQuery.data.lastProcessedAt && (
                <span className="ml-2">
                  Last processed: {formatRelativeAge(managerStatusQuery.data.lastProcessedAt)}
                </span>
              )}
            </div>
          )}

          {/* Loop list */}
          {loopsQuery.isLoading ? (
            <div className="flex items-center justify-center py-4">
              <Loader2 className="h-5 w-5 animate-spin text-muted-foreground" />
              <span className="sr-only">Loading loops...</span>
            </div>
          ) : loops.length === 0 ? (
            <div className="text-center py-4 text-muted-foreground text-sm">
              {showTerminal
                ? "No loops found"
                : "No active loops. Enable 'Show merged/discarded' to see terminal states."}
            </div>
          ) : (
            <div className="space-y-3 max-h-96 overflow-y-auto">
              {loops.map((loop) => (
                <div
                  key={loop.id}
                  className="border border-border rounded-md bg-background"
                >
                  {/* Loop header with badge */}
                  <div className="flex items-center gap-2 px-3 py-2 border-b border-border">
                    <LoopBadge status={loop.status} showPrefix={false} />
                    <span className="font-mono text-xs text-muted-foreground" title={loop.id}>
                      {loop.id.slice(0, 12)}
                    </span>
                    {loop.isPrimary && (
                      <span className="text-xs text-blue-400 font-medium">(primary)</span>
                    )}
                    {loop.startedAt && (
                      <span className="ml-auto text-xs text-muted-foreground flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {formatRelativeAge(loop.startedAt)}
                      </span>
                    )}
                  </div>

                  {/* Loop detail (collapsible) */}
                  <LoopDetail loop={loop} defaultExpanded={false} className="border-none" />

                  {/* Loop actions */}
                  <div className="px-3 py-2 border-t border-border">
                    <LoopActions
                      id={loop.id}
                      status={loop.status}
                      isGitWorkspace={loop.repoRoot !== null && loop.repoRoot !== undefined}
                      callbacks={createActionCallbacks(loop.id)}
                    />
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

MergeQueuePanelComponent.displayName = "MergeQueuePanel";

export const MergeQueuePanel = memo(MergeQueuePanelComponent);
