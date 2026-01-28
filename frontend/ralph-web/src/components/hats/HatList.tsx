/**
 * HatList Component
 *
 * Displays available hats (operational roles) from settings.
 * Shows hat name, description, triggers, and publishes with visual distinction
 * for the currently active hat.
 *
 * Features:
 * - List all defined hats with their metadata
 * - Visual highlighting of the active hat
 * - Click to select/activate a hat
 * - Shows triggersOn and publishes as badges
 * - Dark theme styling consistent with the app
 */

import { trpc } from "../../trpc";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";
import { Badge } from "../ui/badge";
import { Button } from "../ui/button";
import { cn } from "@/lib/utils";
import { Pencil, CheckCircle } from "lucide-react";

interface HatListProps {
  /** Polling interval in ms. Set to 0 to disable. Default: 0 (no polling) */
  pollingInterval?: number;
  /** Callback when a hat is clicked */
  onHatSelect?: (hatKey: string) => void;
  /** Whether to allow changing the active hat on click */
  allowActivation?: boolean;
  /** Optional className for the container */
  className?: string;
}

/**
 * HatList - displays all available hats with their configuration
 */
export function HatList({
  pollingInterval = 0,
  onHatSelect,
  allowActivation = true,
  className,
}: HatListProps) {
  const hatsQuery = trpc.hat.list.useQuery(undefined, {
    refetchInterval: pollingInterval > 0 ? pollingInterval : false,
  });

  const setActiveMutation = trpc.hat.setActive.useMutation({
    onSuccess: () => {
      hatsQuery.refetch();
    },
  });

  const handleActivateHat = (hatKey: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (allowActivation) {
      setActiveMutation.mutate({ key: hatKey });
    }
  };

  const handleEditHat = (hatKey: string, e: React.MouseEvent) => {
    e.stopPropagation();
    onHatSelect?.(hatKey);
  };

  if (hatsQuery.isLoading) {
    return <div className={cn("p-4 text-muted-foreground", className)}>Loading hats...</div>;
  }

  if (hatsQuery.isError) {
    return (
      <div className={cn("p-4", className)}>
        <div className="text-destructive mb-2">Error loading hats: {hatsQuery.error.message}</div>
        <button
          onClick={() => hatsQuery.refetch()}
          className="text-sm text-primary hover:underline"
        >
          Retry
        </button>
      </div>
    );
  }

  const hats = hatsQuery.data ?? [];

  if (hats.length === 0) {
    return (
      <div className={cn("p-4 text-muted-foreground", className)}>
        <p>No hats defined.</p>
        <p className="text-sm mt-1">
          Hats are operational roles that define how the assistant behaves. Configure hats in your
          settings or preset YAML files.
        </p>
      </div>
    );
  }

  return (
    <div className={cn("space-y-3", className)}>
      <div className="flex items-center justify-between px-1">
        <span className="text-sm text-muted-foreground">
          {hats.length} hat{hats.length !== 1 ? "s" : ""} available
        </span>
        <button
          onClick={() => hatsQuery.refetch()}
          disabled={hatsQuery.isFetching}
          className={cn(
            "text-xs px-2 py-1 rounded border border-border",
            "hover:bg-accent transition-colors",
            hatsQuery.isFetching && "opacity-50 cursor-wait"
          )}
        >
          {hatsQuery.isFetching ? "Refreshing..." : "Refresh"}
        </button>
      </div>

      <div className="space-y-2">
        {hats.map((hat) => (
          <Card
            key={hat.key}
            className={cn(
              "transition-all",
              hat.isActive && "border-primary bg-primary/5 ring-1 ring-primary/20",
              setActiveMutation.isPending && "opacity-70 pointer-events-none"
            )}
          >
            <CardHeader className="pb-2">
              <div className="flex items-start justify-between">
                <div>
                  <CardTitle className="text-base flex items-center gap-2">
                    {hat.name}
                    {hat.isActive && (
                      <Badge variant="default" className="text-xs">
                        Active
                      </Badge>
                    )}
                  </CardTitle>
                  <CardDescription className="text-xs mt-0.5">{hat.key}</CardDescription>
                </div>
                <div className="flex items-center gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => handleEditHat(hat.key, e)}
                    className="h-8 px-2"
                  >
                    <Pencil className="h-4 w-4" />
                    <span className="sr-only">Edit</span>
                  </Button>
                  {!hat.isActive && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={(e) => handleActivateHat(hat.key, e)}
                      className="h-8"
                    >
                      <CheckCircle className="h-4 w-4 mr-1" />
                      Activate
                    </Button>
                  )}
                </div>
              </div>
            </CardHeader>
            <CardContent className="pt-0 space-y-3">
              {hat.description && (
                <p className="text-sm text-muted-foreground">{hat.description}</p>
              )}

              {/* Triggers */}
              {hat.triggersOn && hat.triggersOn.length > 0 && (
                <div>
                  <span className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
                    Triggers on
                  </span>
                  <div className="flex flex-wrap gap-1 mt-1">
                    {hat.triggersOn.map((trigger) => (
                      <Badge key={trigger} variant="secondary" className="text-xs font-mono">
                        {trigger}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}

              {/* Publishes */}
              {hat.publishes && hat.publishes.length > 0 && (
                <div>
                  <span className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
                    Publishes
                  </span>
                  <div className="flex flex-wrap gap-1 mt-1">
                    {hat.publishes.map((event) => (
                      <Badge key={event} variant="outline" className="text-xs font-mono">
                        {event}
                      </Badge>
                    ))}
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        ))}
      </div>
    </div>
  );
}
