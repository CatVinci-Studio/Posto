import { cn } from "@/lib/utils";

interface EmptyStateProps {
  icon?: React.ReactNode;
  title: string;
  subtitle?: string;
  cta?: React.ReactNode;
  className?: string;
}

export function EmptyState({ icon, title, subtitle, cta, className }: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center gap-3 py-20 text-center",
        className
      )}
    >
      {icon && (
        <div className="text-5xl leading-none select-none" aria-hidden="true">
          {icon}
        </div>
      )}
      <p className="text-base font-medium text-foreground">{title}</p>
      {subtitle && (
        <p className="max-w-xs text-sm text-muted-foreground">{subtitle}</p>
      )}
      {cta && <div className="mt-2">{cta}</div>}
    </div>
  );
}
