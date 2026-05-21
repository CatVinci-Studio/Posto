import { useTranslation } from "react-i18next";
import { cn } from "@/lib/utils";
import type { AgentCategory } from "@/types/email";

const CATEGORY_STYLES: Record<AgentCategory, string> = {
  urgent:
    "bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-300",
  work:
    "bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-300",
  personal:
    "bg-emerald-100 text-emerald-700 dark:bg-emerald-900/30 dark:text-emerald-300",
  newsletter:
    "bg-slate-100 text-slate-600 dark:bg-slate-800/50 dark:text-slate-400",
  transactional:
    "bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-300",
  social:
    "bg-pink-100 text-pink-700 dark:bg-pink-900/30 dark:text-pink-300",
  spam:
    "bg-gray-100 text-gray-400 dark:bg-gray-800/40 dark:text-gray-500",
};

interface CategoryChipProps {
  category: AgentCategory;
  className?: string;
}

export function CategoryChip({ category, className }: CategoryChipProps) {
  const { t } = useTranslation();
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium leading-none",
        CATEGORY_STYLES[category],
        className
      )}
    >
      {t(`inbox.categories.${category}`)}
    </span>
  );
}
