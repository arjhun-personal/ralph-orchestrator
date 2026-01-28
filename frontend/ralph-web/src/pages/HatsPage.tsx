/**
 * Hats Page
 *
 * Configuration interface for Ralph persona presets (hats).
 * Supports listing all hats, editing existing hats, and creating new ones.
 *
 * Features:
 * - View all hat presets with active status
 * - Click to select/activate a hat
 * - Edit button to modify hat configuration
 * - Create new hat button
 * - Delete hat with confirmation
 */

import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { HatList } from "@/components/hats/HatList";
import { HatEditor } from "@/components/hats/HatEditor";
import { Plus } from "lucide-react";

type ViewMode = "list" | "edit" | "create";

export function HatsPage() {
  const [viewMode, setViewMode] = useState<ViewMode>("list");
  const [selectedHatKey, setSelectedHatKey] = useState<string | null>(null);

  const handleEditHat = (hatKey: string) => {
    setSelectedHatKey(hatKey);
    setViewMode("edit");
  };

  const handleCreateHat = () => {
    setSelectedHatKey(null);
    setViewMode("create");
  };

  const handleSaveComplete = () => {
    setViewMode("list");
    setSelectedHatKey(null);
  };

  const handleCancel = () => {
    setViewMode("list");
    setSelectedHatKey(null);
  };

  return (
    <>
      {/* Page header */}
      <header className="mb-6 flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold tracking-tight">Hats</h1>
          <p className="text-muted-foreground text-sm mt-1">Configure persona presets for Ralph</p>
        </div>
        <div className="flex items-center gap-3">
          {viewMode === "list" && (
            <Button onClick={handleCreateHat}>
              <Plus className="h-4 w-4 mr-2" />
              New Hat
            </Button>
          )}
          <Badge variant="secondary">v0.1.0</Badge>
        </div>
      </header>

      {/* Content */}
      {viewMode === "list" ? (
        <HatList onHatSelect={handleEditHat} allowActivation={true} />
      ) : (
        <HatEditor
          hatKey={viewMode === "edit" ? selectedHatKey : null}
          onSave={handleSaveComplete}
          onCancel={handleCancel}
        />
      )}
    </>
  );
}
