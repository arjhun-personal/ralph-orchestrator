/**
 * HatEditor Component
 *
 * Form for editing hat presets with fields for name, description,
 * triggersOn, publishes, and instructions. Includes validation,
 * save/cancel buttons, and a live preview panel.
 *
 * Features:
 * - Form validation with user-friendly error messages
 * - Tag input for triggersOn and publishes arrays
 * - Live preview of hat configuration
 * - Create new hat or edit existing hat mode
 */

import { useState, useCallback, useMemo, useEffect } from "react";
import { trpc } from "../../trpc";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import { Textarea } from "../ui/textarea";
import { Label } from "../ui/label";
import { Badge } from "../ui/badge";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "../ui/card";
import { cn } from "@/lib/utils";
import { X, Plus, Save, ArrowLeft, AlertCircle } from "lucide-react";

/** Form data for hat editing */
interface HatFormData {
  key: string;
  name: string;
  description: string;
  triggersOn: string[];
  publishes: string[];
  instructions: string;
}

/** Validation errors for form fields */
interface FormErrors {
  key?: string;
  name?: string;
  description?: string;
  triggersOn?: string;
  publishes?: string;
}

interface HatEditorProps {
  /** Hat key to edit (null for new hat) */
  hatKey: string | null;
  /** Callback when save is successful */
  onSave: () => void;
  /** Callback when cancel is clicked */
  onCancel: () => void;
  /** Optional className */
  className?: string;
}

/** Generate a key from a name (kebab-case) */
function generateKey(name: string): string {
  return name
    .toLowerCase()
    .trim()
    .replace(/[^a-z0-9\s-]/g, "")
    .replace(/\s+/g, "-")
    .replace(/-+/g, "-");
}

/** Validate form data */
function validateForm(data: HatFormData, isNew: boolean): FormErrors {
  const errors: FormErrors = {};

  if (isNew && !data.key.trim()) {
    errors.key = "Key is required";
  } else if (isNew && !/^[a-z0-9-]+$/.test(data.key)) {
    errors.key = "Key must be lowercase letters, numbers, and hyphens only";
  }

  if (!data.name.trim()) {
    errors.name = "Name is required";
  }

  if (!data.description.trim()) {
    errors.description = "Description is required";
  }

  return errors;
}

/**
 * TagInput - inline tag editor for arrays of strings
 */
function TagInput({
  value,
  onChange,
  placeholder,
  label,
  error,
}: {
  value: string[];
  onChange: (tags: string[]) => void;
  placeholder: string;
  label: string;
  error?: string;
}) {
  const [inputValue, setInputValue] = useState("");

  const addTag = useCallback(() => {
    const tag = inputValue.trim();
    if (tag && !value.includes(tag)) {
      onChange([...value, tag]);
      setInputValue("");
    }
  }, [inputValue, value, onChange]);

  const removeTag = useCallback(
    (tagToRemove: string) => {
      onChange(value.filter((tag) => tag !== tagToRemove));
    },
    [value, onChange]
  );

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent) => {
      if (e.key === "Enter") {
        e.preventDefault();
        addTag();
      } else if (e.key === "Backspace" && !inputValue && value.length > 0) {
        // Remove last tag on backspace when input is empty
        removeTag(value[value.length - 1]);
      }
    },
    [addTag, inputValue, value, removeTag]
  );

  return (
    <div className="space-y-2">
      <Label className={error ? "text-destructive" : ""}>{label}</Label>
      <div
        className={cn(
          "flex flex-wrap gap-1.5 p-2 rounded-md border border-input bg-transparent",
          "focus-within:ring-1 focus-within:ring-ring",
          error && "border-destructive"
        )}
      >
        {value.map((tag) => (
          <Badge
            key={tag}
            variant="secondary"
            className="text-xs font-mono flex items-center gap-1"
          >
            {tag}
            <button
              type="button"
              onClick={() => removeTag(tag)}
              className="hover:text-destructive transition-colors"
            >
              <X className="h-3 w-3" />
            </button>
          </Badge>
        ))}
        <input
          type="text"
          value={inputValue}
          onChange={(e) => setInputValue(e.target.value)}
          onKeyDown={handleKeyDown}
          onBlur={addTag}
          placeholder={value.length === 0 ? placeholder : ""}
          className="flex-1 min-w-[120px] bg-transparent outline-none text-sm placeholder:text-muted-foreground"
        />
        <Button
          type="button"
          variant="ghost"
          size="sm"
          onClick={addTag}
          disabled={!inputValue.trim()}
          className="h-6 px-1.5"
        >
          <Plus className="h-3 w-3" />
        </Button>
      </div>
      {error && (
        <p className="text-xs text-destructive flex items-center gap-1">
          <AlertCircle className="h-3 w-3" />
          {error}
        </p>
      )}
    </div>
  );
}

/**
 * HatPreview - shows a live preview of the hat configuration
 */
function HatPreview({ data }: { data: HatFormData }) {
  return (
    <Card className="bg-muted/30">
      <CardHeader className="pb-2">
        <CardTitle className="text-sm font-medium text-muted-foreground uppercase tracking-wide">
          Preview
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-3">
        <div>
          <span className="text-xs text-muted-foreground">Key</span>
          <p className="font-mono text-sm">{data.key || "—"}</p>
        </div>
        <div>
          <span className="text-xs text-muted-foreground">Name</span>
          <p className="font-medium">{data.name || "—"}</p>
        </div>
        <div>
          <span className="text-xs text-muted-foreground">Description</span>
          <p className="text-sm text-muted-foreground">{data.description || "—"}</p>
        </div>

        {data.triggersOn.length > 0 && (
          <div>
            <span className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
              Triggers on
            </span>
            <div className="flex flex-wrap gap-1 mt-1">
              {data.triggersOn.map((trigger) => (
                <Badge key={trigger} variant="secondary" className="text-xs font-mono">
                  {trigger}
                </Badge>
              ))}
            </div>
          </div>
        )}

        {data.publishes.length > 0 && (
          <div>
            <span className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
              Publishes
            </span>
            <div className="flex flex-wrap gap-1 mt-1">
              {data.publishes.map((event) => (
                <Badge key={event} variant="outline" className="text-xs font-mono">
                  {event}
                </Badge>
              ))}
            </div>
          </div>
        )}

        {data.instructions && (
          <div>
            <span className="text-xs text-muted-foreground font-medium uppercase tracking-wide">
              Instructions
            </span>
            <pre className="mt-1 text-xs font-mono whitespace-pre-wrap text-muted-foreground bg-background/50 p-2 rounded">
              {data.instructions.slice(0, 200)}
              {data.instructions.length > 200 && "..."}
            </pre>
          </div>
        )}
      </CardContent>
    </Card>
  );
}

/**
 * HatEditor - main form component
 */
export function HatEditor({ hatKey, onSave, onCancel, className }: HatEditorProps) {
  const isNew = hatKey === null;

  // Query existing hat data if editing
  const hatQuery = trpc.hat.get.useQuery(
    { key: hatKey! },
    {
      enabled: !isNew && !!hatKey,
    }
  );

  // Default empty form data
  const emptyFormData: HatFormData = useMemo(
    () => ({
      key: "",
      name: "",
      description: "",
      triggersOn: [],
      publishes: [],
      instructions: "",
    }),
    []
  );

  // Form state
  const [formData, setFormData] = useState<HatFormData>(emptyFormData);
  const [errors, setErrors] = useState<FormErrors>({});
  const [autoGenerateKey, setAutoGenerateKey] = useState(true);

  // Sync form data when hat data loads
  // Using useEffect is the correct pattern for syncing external data to local state
  /* eslint-disable react-hooks/set-state-in-effect */
  useEffect(() => {
    if (hatQuery.data) {
      setFormData({
        key: hatQuery.data.key,
        name: hatQuery.data.name,
        description: hatQuery.data.description,
        triggersOn: hatQuery.data.triggersOn || [],
        publishes: hatQuery.data.publishes || [],
        instructions: hatQuery.data.instructions || "",
      });
      setAutoGenerateKey(false);
    } else if (isNew) {
      setFormData(emptyFormData);
      setAutoGenerateKey(true);
    }
  }, [hatQuery.data, isNew, emptyFormData]);
  /* eslint-enable react-hooks/set-state-in-effect */

  // Save mutation
  const saveMutation = trpc.hat.save.useMutation({
    onSuccess: () => {
      onSave();
    },
  });

  // Update field handler
  const updateField = useCallback(
    <K extends keyof HatFormData>(field: K, value: HatFormData[K]) => {
      setFormData((prev) => {
        const updated = { ...prev, [field]: value };
        // Auto-generate key from name if creating new hat
        if (field === "name" && isNew && autoGenerateKey) {
          updated.key = generateKey(value as string);
        }
        return updated;
      });
      // Clear error when field is updated
      if (errors[field as keyof FormErrors]) {
        setErrors((prev) => ({ ...prev, [field]: undefined }));
      }
    },
    [isNew, autoGenerateKey, errors]
  );

  // Handle form submission
  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();

      const validationErrors = validateForm(formData, isNew);
      if (Object.keys(validationErrors).length > 0) {
        setErrors(validationErrors);
        return;
      }

      saveMutation.mutate({
        key: formData.key,
        name: formData.name,
        description: formData.description,
        triggersOn: formData.triggersOn,
        publishes: formData.publishes,
        instructions: formData.instructions || undefined,
      });
    },
    [formData, isNew, saveMutation]
  );

  // Loading state for edit mode
  if (!isNew && hatQuery.isLoading) {
    return <div className={cn("p-4 text-muted-foreground", className)}>Loading hat...</div>;
  }

  // Error state for edit mode
  if (!isNew && hatQuery.isError) {
    return (
      <div className={cn("p-4", className)}>
        <div className="text-destructive mb-2">Error loading hat: {hatQuery.error.message}</div>
        <Button variant="outline" onClick={onCancel}>
          <ArrowLeft className="h-4 w-4 mr-2" />
          Back to list
        </Button>
      </div>
    );
  }

  return (
    <div className={cn("grid grid-cols-1 lg:grid-cols-2 gap-6", className)}>
      {/* Form */}
      <form onSubmit={handleSubmit} className="space-y-6">
        <Card>
          <CardHeader>
            <CardTitle>{isNew ? "Create New Hat" : `Edit: ${formData.name}`}</CardTitle>
            <CardDescription>
              {isNew ? "Define a new operational role for Ralph" : "Modify the hat configuration"}
            </CardDescription>
          </CardHeader>
          <CardContent className="space-y-4">
            {/* Key field (only editable for new hats) */}
            <div className="space-y-2">
              <Label htmlFor="key" className={errors.key ? "text-destructive" : ""}>
                Key
              </Label>
              <Input
                id="key"
                value={formData.key}
                onChange={(e) => {
                  updateField("key", e.target.value);
                  setAutoGenerateKey(false);
                }}
                disabled={!isNew}
                placeholder="e.g., builder, planner, validator"
                className={cn(
                  "font-mono",
                  errors.key && "border-destructive",
                  !isNew && "opacity-50"
                )}
              />
              {errors.key && (
                <p className="text-xs text-destructive flex items-center gap-1">
                  <AlertCircle className="h-3 w-3" />
                  {errors.key}
                </p>
              )}
              {isNew && (
                <p className="text-xs text-muted-foreground">
                  Unique identifier for the hat (auto-generated from name)
                </p>
              )}
            </div>

            {/* Name field */}
            <div className="space-y-2">
              <Label htmlFor="name" className={errors.name ? "text-destructive" : ""}>
                Name
              </Label>
              <Input
                id="name"
                value={formData.name}
                onChange={(e) => updateField("name", e.target.value)}
                placeholder="e.g., Builder, Planner, Validator"
                className={errors.name ? "border-destructive" : ""}
              />
              {errors.name && (
                <p className="text-xs text-destructive flex items-center gap-1">
                  <AlertCircle className="h-3 w-3" />
                  {errors.name}
                </p>
              )}
            </div>

            {/* Description field */}
            <div className="space-y-2">
              <Label htmlFor="description" className={errors.description ? "text-destructive" : ""}>
                Description
              </Label>
              <Textarea
                id="description"
                value={formData.description}
                onChange={(e) => updateField("description", e.target.value)}
                placeholder="Brief description of what this hat does..."
                rows={2}
                className={errors.description ? "border-destructive" : ""}
              />
              {errors.description && (
                <p className="text-xs text-destructive flex items-center gap-1">
                  <AlertCircle className="h-3 w-3" />
                  {errors.description}
                </p>
              )}
            </div>

            {/* Triggers On */}
            <TagInput
              value={formData.triggersOn}
              onChange={(tags) => updateField("triggersOn", tags)}
              placeholder="Add event triggers (e.g., build.task)"
              label="Triggers On"
              error={errors.triggersOn}
            />

            {/* Publishes */}
            <TagInput
              value={formData.publishes}
              onChange={(tags) => updateField("publishes", tags)}
              placeholder="Add events to publish (e.g., build.done)"
              label="Publishes"
              error={errors.publishes}
            />

            {/* Instructions */}
            <div className="space-y-2">
              <Label htmlFor="instructions">Instructions (Optional)</Label>
              <Textarea
                id="instructions"
                value={formData.instructions}
                onChange={(e) => updateField("instructions", e.target.value)}
                placeholder="Additional instructions for this hat's behavior..."
                rows={4}
                className="font-mono text-sm"
              />
              <p className="text-xs text-muted-foreground">
                Markdown-formatted instructions injected into the prompt
              </p>
            </div>
          </CardContent>
        </Card>

        {/* Actions */}
        <div className="flex justify-between">
          <Button type="button" variant="outline" onClick={onCancel}>
            <ArrowLeft className="h-4 w-4 mr-2" />
            Cancel
          </Button>
          <Button type="submit" disabled={saveMutation.isPending}>
            <Save className="h-4 w-4 mr-2" />
            {saveMutation.isPending ? "Saving..." : "Save Hat"}
          </Button>
        </div>

        {/* Save error */}
        {saveMutation.isError && (
          <div className="p-3 rounded-md bg-destructive/10 border border-destructive/20 text-destructive text-sm">
            <AlertCircle className="h-4 w-4 inline mr-2" />
            {saveMutation.error.message}
          </div>
        )}
      </form>

      {/* Preview */}
      <div className="lg:sticky lg:top-4">
        <HatPreview data={formData} />
      </div>
    </div>
  );
}
