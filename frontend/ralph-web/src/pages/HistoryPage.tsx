/**
 * History Page
 *
 * View of past task executions and their results.
 * Placeholder for future implementation.
 */

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { History } from "lucide-react";

export function HistoryPage() {
  return (
    <>
      {/* Page header */}
      <header className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">History</h1>
          <p className="text-muted-foreground text-sm mt-1">View past task executions</p>
        </div>
        <Badge variant="secondary">v0.1.0</Badge>
      </header>

      {/* Content */}
      <Card>
        <CardHeader>
          <CardTitle>Task History</CardTitle>
          <CardDescription>View past task executions</CardDescription>
        </CardHeader>
        <CardContent>
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <History className="h-12 w-12 text-muted-foreground mb-4" />
            <p className="text-muted-foreground">History view coming in a future update</p>
          </div>
        </CardContent>
      </Card>
    </>
  );
}
