import * as React from "react";
import { X } from "lucide-react";
import { cn } from "@/lib/utils";

interface AddressInputProps {
  label: string;
  value: string[];
  onChange: (addresses: string[]) => void;
  placeholder?: string;
  className?: string;
  id?: string;
}

function isValidEmailLike(value: string): boolean {
  return value.includes("@") && value.includes(".");
}

/**
 * Chip-style email tag input.
 * Press Enter or comma to confirm an address; click × to remove.
 * Pastes are split on comma and newline automatically.
 */
export function AddressInput({
  label,
  value,
  onChange,
  placeholder,
  className,
  id,
}: AddressInputProps) {
  const [inputValue, setInputValue] = React.useState("");
  const inputRef = React.useRef<HTMLInputElement>(null);
  const inputId = id ?? label.toLowerCase().replace(/\s+/g, "-");

  const addAddress = (raw: string) => {
    const trimmed = raw.trim().replace(/,+$/, "");
    if (!trimmed) return;
    const newAddr = trimmed.toLowerCase();
    if (!value.includes(newAddr)) {
      onChange([...value, newAddr]);
    }
    setInputValue("");
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === "Enter" || e.key === ",") {
      e.preventDefault();
      addAddress(inputValue);
    } else if (e.key === "Backspace" && inputValue === "" && value.length > 0) {
      onChange(value.slice(0, -1));
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const raw = e.target.value;
    // If user typed a comma, treat text before it as completed chip
    if (raw.endsWith(",")) {
      addAddress(raw.slice(0, -1));
    } else {
      setInputValue(raw);
    }
  };

  const handlePaste = (e: React.ClipboardEvent<HTMLInputElement>) => {
    e.preventDefault();
    const text = e.clipboardData.getData("text");
    const parts = text.split(/[,\n;]+/).map((s) => s.trim()).filter(Boolean);
    const unique = parts.filter((p) => !value.includes(p.toLowerCase()));
    if (unique.length > 0) {
      onChange([...value, ...unique.map((p) => p.toLowerCase())]);
    }
  };

  const removeAt = (idx: number) => {
    onChange(value.filter((_, i) => i !== idx));
  };

  return (
    <div className={cn("flex flex-col gap-1.5", className)}>
      <label htmlFor={inputId} className="text-sm font-medium text-muted-foreground w-12 shrink-0">
        {label}
      </label>
      <div
        className={cn(
          "flex min-h-[2.5rem] flex-wrap items-center gap-1.5 rounded-md border border-border bg-background px-3 py-1.5",
          "focus-within:ring-2 focus-within:ring-ring focus-within:ring-offset-2 cursor-text"
        )}
        onClick={() => inputRef.current?.focus()}
      >
        {value.map((addr, idx) => (
          <span
            key={`${addr}-${idx}`}
            className={cn(
              "inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium",
              isValidEmailLike(addr)
                ? "bg-primary/10 text-primary"
                : "bg-destructive/10 text-destructive"
            )}
          >
            {addr}
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                removeAt(idx);
              }}
              className="ml-0.5 rounded-full p-0.5 hover:bg-black/10 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring"
              aria-label={`Remove ${addr}`}
            >
              <X className="h-2.5 w-2.5" />
            </button>
          </span>
        ))}
        <input
          ref={inputRef}
          id={inputId}
          type="text"
          value={inputValue}
          onChange={handleChange}
          onKeyDown={handleKeyDown}
          onPaste={handlePaste}
          onBlur={() => addAddress(inputValue)}
          placeholder={value.length === 0 ? placeholder : undefined}
          className="min-w-[8rem] flex-1 bg-transparent text-sm text-foreground placeholder:text-muted-foreground focus:outline-none"
        />
      </div>
    </div>
  );
}
