import * as React from "react";
import { useTranslation } from "react-i18next";
import { Pin, PinOff, Pencil, Trash2, Plus, Download } from "lucide-react";
import { Card, CardHeader, CardContent } from "@/components/ui/Card";
import { Button } from "@/components/ui/Button";
import { Dialog, DialogFooter } from "@/components/ui/Dialog";
import { Input } from "@/components/ui/Input";
import { Select } from "@/components/ui/Select";
import { call } from "@/lib/tauri";
import { cn } from "@/lib/utils";
import type { MemoryEntry, MemoryType } from "@/types/settings";

const TYPE_COLORS: Record<MemoryType, string> = {
  contact: "bg-blue-100 text-blue-700 dark:bg-blue-900/40 dark:text-blue-300",
  preference: "bg-purple-100 text-purple-700 dark:bg-purple-900/40 dark:text-purple-300",
  project: "bg-green-100 text-green-700 dark:bg-green-900/40 dark:text-green-300",
  rule: "bg-amber-100 text-amber-700 dark:bg-amber-900/40 dark:text-amber-300",
  fact: "bg-gray-100 text-gray-700 dark:bg-gray-800/60 dark:text-gray-300",
};

const ALL_TYPES: MemoryType[] = ["contact", "preference", "project", "rule", "fact"];

// ---------------------------------------------------------------------------
// Sub-components
// ---------------------------------------------------------------------------
function ImportanceBar({ value }: { value: number }) {
  return (
    <div className="flex items-center gap-1.5">
      <div className="flex-1 h-1.5 rounded-full bg-muted overflow-hidden">
        <div
          className="h-full rounded-full bg-primary/70"
          style={{ width: `${Math.round(value * 100)}%` }}
        />
      </div>
      <span className="text-xs text-muted-foreground w-7 text-right">
        {Math.round(value * 100)}%
      </span>
    </div>
  );
}

interface MemoryCardProps {
  entry: MemoryEntry;
  onPin: (id: number) => void;
  onDelete: (id: number) => void;
  onEdit: (entry: MemoryEntry) => void;
}

function MemoryCard({ entry, onPin, onDelete, onEdit }: MemoryCardProps) {
  const { t } = useTranslation();
  return (
    <div className={cn(
      "rounded-lg border border-border bg-background p-4 space-y-2.5 transition-shadow hover:shadow-sm",
      entry.pinned ? "border-primary/30 bg-primary/5" : ""
    )}>
      <div className="flex items-start justify-between gap-2">
        <div className="flex items-center gap-2 flex-wrap">
          <span className={cn(
            "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium",
            TYPE_COLORS[entry.type]
          )}>
            {entry.type}
          </span>
          {entry.scope !== "global" && (
            <span className="text-xs text-muted-foreground bg-muted px-2 py-0.5 rounded-full">
              {entry.scope}
            </span>
          )}
          {entry.key && (
            <span className="text-xs font-mono text-muted-foreground">#{entry.key}</span>
          )}
        </div>
        <div className="flex items-center gap-1 shrink-0">
          <button
            onClick={() => onPin(entry.id)}
            className="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
            title={entry.pinned ? t("settings.memory.unpin") : t("settings.memory.pin")}
          >
            {entry.pinned ? <Pin className="h-3.5 w-3.5 fill-current" /> : <PinOff className="h-3.5 w-3.5" />}
          </button>
          <button
            onClick={() => onEdit(entry)}
            className="p-1 rounded text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
          >
            <Pencil className="h-3.5 w-3.5" />
          </button>
          <button
            onClick={() => onDelete(entry.id)}
            className="p-1 rounded text-muted-foreground hover:text-red-500 hover:bg-muted transition-colors"
          >
            <Trash2 className="h-3.5 w-3.5" />
          </button>
        </div>
      </div>
      <p className="text-sm text-foreground leading-relaxed">{entry.content}</p>
      <div className="space-y-1">
        <div className="flex items-center justify-between">
          <span className="text-xs text-muted-foreground">{t("settings.memory.importance")}</span>
        </div>
        <ImportanceBar value={entry.importance} />
      </div>
      <p className="text-xs text-muted-foreground">
        Used {entry.use_count}x · Created {new Date(entry.created_at).toLocaleDateString()}
      </p>
    </div>
  );
}

// ---------------------------------------------------------------------------
// Add/Edit Dialog
// ---------------------------------------------------------------------------
interface AddMemoryDialogProps {
  open: boolean;
  initial?: MemoryEntry | null;
  onClose: () => void;
  onSave: (data: Pick<MemoryEntry, "type" | "scope" | "content" | "key">) => void;
}

function AddMemoryDialog({ open, initial, onClose, onSave }: AddMemoryDialogProps) {
  const { t } = useTranslation();
  const [type, setType] = React.useState<MemoryType>(initial?.type ?? "fact");
  const [scope, setScope] = React.useState(initial?.scope ?? "global");
  const [content, setContent] = React.useState(initial?.content ?? "");
  const [key, setKey] = React.useState(initial?.key ?? "");

  React.useEffect(() => {
    if (initial) {
      setType(initial.type);
      setScope(initial.scope);
      setContent(initial.content);
      setKey(initial.key ?? "");
    } else {
      setType("fact");
      setScope("global");
      setContent("");
      setKey("");
    }
  }, [initial, open]);

  return (
    <Dialog
      open={open}
      onClose={onClose}
      title={initial ? "Edit Memory" : t("settings.memory.add")}
    >
      <div className="space-y-3">
        <Select
          label="Type"
          value={type}
          onChange={(e) => setType(e.target.value as MemoryType)}
        >
          {ALL_TYPES.map((tp) => (
            <option key={tp} value={tp}>{tp}</option>
          ))}
        </Select>
        <Input
          label="Scope"
          value={scope}
          onChange={(e) => setScope(e.target.value)}
          placeholder="global / account:1 / contact:email@example.com"
        />
        <Input
          label="Key (optional)"
          value={key}
          onChange={(e) => setKey(e.target.value)}
          placeholder="e.g. project-name"
        />
        <div className="flex flex-col gap-1.5">
          <label className="text-sm font-medium">Content</label>
          <textarea
            value={content}
            onChange={(e) => setContent(e.target.value)}
            rows={3}
            className="w-full rounded-md border border-border bg-background px-3 py-2 text-sm text-foreground placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring resize-none"
            placeholder="What should the AI remember?"
          />
        </div>
      </div>
      <DialogFooter>
        <Button variant="outline" size="sm" onClick={onClose}>Cancel</Button>
        <Button
          variant="default"
          size="sm"
          disabled={!content.trim()}
          onClick={() => {
            onSave({ type, scope, content, key: key || undefined });
            onClose();
          }}
        >
          Save
        </Button>
      </DialogFooter>
    </Dialog>
  );
}

// ---------------------------------------------------------------------------
// MemoryTab
// ---------------------------------------------------------------------------
export function MemoryTab() {
  const { t } = useTranslation();
  const [memories, setMemories] = React.useState<MemoryEntry[]>([]);
  const [typeFilter, setTypeFilter] = React.useState<MemoryType | "all">("all");
  const [search, setSearch] = React.useState("");
  const [dialogOpen, setDialogOpen] = React.useState(false);
  const [editTarget, setEditTarget] = React.useState<MemoryEntry | null>(null);
  const [confirmDeleteId, setConfirmDeleteId] = React.useState<number | null>(null);

  const [loadError, setLoadError] = React.useState<string | null>(null);
  React.useEffect(() => {
    call<MemoryEntry[]>("list_memories", {})
      .then((data) => {
        setMemories(data);
        setLoadError(null);
      })
      .catch((err) => {
        setMemories([]);
        setLoadError(err instanceof Error ? err.message : String(err));
      });
  }, []);

  const filtered = memories.filter((m) => {
    if (typeFilter !== "all" && m.type !== typeFilter) return false;
    if (search && !m.content.toLowerCase().includes(search.toLowerCase())) return false;
    return true;
  });

  // Group pinned first, then by type
  const sorted = [...filtered].sort((a, b) => {
    if (a.pinned !== b.pinned) return b.pinned - a.pinned;
    return a.type.localeCompare(b.type);
  });

  function handlePin(id: number) {
    const target = memories.find((m) => m.id === id);
    if (!target) return;
    const newPinned = target.pinned ? 0 : 1;
    call("pin_memory", { id, pinned: newPinned }).catch(() => {});
    setMemories((prev) =>
      prev.map((m) => (m.id === id ? { ...m, pinned: newPinned as 0 | 1 } : m))
    );
  }

  function handleDelete(id: number) {
    setConfirmDeleteId(id);
  }

  function confirmDelete() {
    if (confirmDeleteId == null) return;
    call("delete_memory", { id: confirmDeleteId }).catch(() => {});
    setMemories((prev) => prev.filter((m) => m.id !== confirmDeleteId));
    setConfirmDeleteId(null);
  }

  function handleEdit(entry: MemoryEntry) {
    setEditTarget(entry);
    setDialogOpen(true);
  }

  function handleSave(data: Pick<MemoryEntry, "type" | "scope" | "content" | "key">) {
    if (editTarget) {
      const updated = { ...editTarget, ...data };
      call("update_memory", updated).catch(() => {});
      setMemories((prev) => prev.map((m) => (m.id === editTarget.id ? updated : m)));
    } else {
      const newEntry: MemoryEntry = {
        id: Date.now(),
        ...data,
        importance: 0.5,
        pinned: 0,
        created_at: Date.now(),
        use_count: 0,
      };
      call("add_memory_manual", { ...newEntry } as unknown as Record<string, unknown>).catch(() => {});
      setMemories((prev) => [newEntry, ...prev]);
    }
    setEditTarget(null);
  }

  function handleExport() {
    const lines: string[] = ["# Posto Long-Term Memory Export", ""];
    const byType = ALL_TYPES.reduce((acc, t) => {
      acc[t] = memories.filter((m) => m.type === t);
      return acc;
    }, {} as Record<MemoryType, MemoryEntry[]>);

    for (const type of ALL_TYPES) {
      const group = byType[type];
      if (!group.length) continue;
      lines.push(`## ${type.charAt(0).toUpperCase() + type.slice(1)}`);
      for (const m of group) {
        lines.push(`- ${m.pinned ? "📌 " : ""}${m.content} *(scope: ${m.scope}, importance: ${Math.round(m.importance * 100)}%)*`);
      }
      lines.push("");
    }

    const blob = new Blob([lines.join("\n")], { type: "text/markdown" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = "posto-memory.md";
    a.click();
    URL.revokeObjectURL(url);
  }

  return (
    <div className="space-y-6">
      {/* Filters */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div>
              <h2 className="text-base font-semibold">{t("settings.tabs.memory")}</h2>
              <p className="text-xs text-muted-foreground">{memories.length} memories stored</p>
            </div>
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="sm"
                onClick={handleExport}
                className="gap-1.5"
              >
                <Download className="h-4 w-4" />
                {t("settings.memory.export")}
              </Button>
              <Button
                variant="default"
                size="sm"
                onClick={() => { setEditTarget(null); setDialogOpen(true); }}
                className="gap-1.5"
              >
                <Plus className="h-4 w-4" />
                {t("settings.memory.add")}
              </Button>
            </div>
          </div>
        </CardHeader>
        <CardContent>
          <div className="flex flex-wrap gap-3">
            {/* Type filter */}
            <Select
              value={typeFilter}
              onChange={(e) => setTypeFilter(e.target.value as MemoryType | "all")}
              className="w-40"
            >
              <option value="all">{t("settings.memory.filter_type")}: All</option>
              {ALL_TYPES.map((t) => (
                <option key={t} value={t}>{t.charAt(0).toUpperCase() + t.slice(1)}</option>
              ))}
            </Select>
            {/* Search */}
            <div className="flex-1 min-w-[180px]">
              <Input
                placeholder={t("settings.memory.search")}
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </div>
          </div>
        </CardContent>
      </Card>

      {loadError && (
        <div className="rounded-md border border-destructive/40 bg-destructive/10 px-3 py-2 text-xs text-destructive">
          {loadError}
        </div>
      )}

      {/* Memory list */}
      {sorted.length === 0 ? (
        <Card>
          <CardContent>
            <p className="text-sm text-muted-foreground text-center py-8">
              {t("settings.memory.no_memories")}
            </p>
          </CardContent>
        </Card>
      ) : (
        <div className="grid grid-cols-1 gap-3">
          {sorted.map((entry) => (
            <MemoryCard
              key={entry.id}
              entry={entry}
              onPin={handlePin}
              onDelete={handleDelete}
              onEdit={handleEdit}
            />
          ))}
        </div>
      )}

      {/* Delete confirm */}
      <Dialog
        open={confirmDeleteId != null}
        onClose={() => setConfirmDeleteId(null)}
        title="Delete Memory"
      >
        <p className="text-sm text-foreground">{t("settings.memory.confirm_delete")}</p>
        <DialogFooter>
          <Button variant="outline" size="sm" onClick={() => setConfirmDeleteId(null)}>
            Cancel
          </Button>
          <Button
            size="sm"
            className="bg-red-600 hover:bg-red-700 text-white"
            onClick={confirmDelete}
          >
            Delete
          </Button>
        </DialogFooter>
      </Dialog>

      {/* Add/Edit dialog */}
      <AddMemoryDialog
        open={dialogOpen}
        initial={editTarget}
        onClose={() => { setDialogOpen(false); setEditTarget(null); }}
        onSave={handleSave}
      />
    </div>
  );
}
