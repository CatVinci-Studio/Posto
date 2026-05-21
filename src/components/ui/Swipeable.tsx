import { useCallback, useRef, useState, type ReactNode, type PointerEvent } from "react";
import { cn } from "@/lib/utils";

interface SwipeAction {
  /** Visible label (often hidden behind icon-only display). */
  label: string;
  /** Icon to render. */
  icon: ReactNode;
  /** Background color via tailwind class, e.g. "bg-destructive". */
  color: string;
  onAction: () => void;
}

interface SwipeableProps {
  children: ReactNode;
  /** Actions revealed by swiping right→left (left side of card hidden). */
  leftActions?: SwipeAction[];
  /** Actions revealed by swiping left→right. */
  rightActions?: SwipeAction[];
  className?: string;
}

const SWIPE_THRESHOLD = 60;
const MAX_REVEAL = 220;

/**
 * iOS-style swipe-to-reveal row. Each action sits behind the row and slides
 * into view as the user drags. Releasing past `SWIPE_THRESHOLD` triggers the
 * last (outermost) action; releasing before then snaps back. Designed for
 * touch but the pointer events also support trackpad / mouse drags.
 */
export function Swipeable({
  children,
  leftActions = [],
  rightActions = [],
  className,
}: SwipeableProps) {
  const [offset, setOffset] = useState(0);
  const startX = useRef<number | null>(null);
  const startOffset = useRef(0);

  const onPointerDown = useCallback(
    (e: PointerEvent<HTMLDivElement>) => {
      if (e.pointerType === "mouse" && e.button !== 0) return;
      startX.current = e.clientX;
      startOffset.current = offset;
      (e.target as Element).setPointerCapture(e.pointerId);
    },
    [offset],
  );

  const onPointerMove = useCallback(
    (e: PointerEvent<HTMLDivElement>) => {
      if (startX.current === null) return;
      const delta = e.clientX - startX.current;
      let next = startOffset.current + delta;
      // Clamp by available actions on each side.
      const maxLeft = rightActions.length > 0 ? MAX_REVEAL : 0;
      const maxRight = leftActions.length > 0 ? -MAX_REVEAL : 0;
      next = Math.max(maxRight, Math.min(maxLeft, next));
      setOffset(next);
    },
    [leftActions.length, rightActions.length],
  );

  const onPointerUp = useCallback(() => {
    if (startX.current === null) return;
    startX.current = null;

    if (offset <= -SWIPE_THRESHOLD && leftActions.length > 0) {
      // Outermost (last) left action fires.
      leftActions[leftActions.length - 1].onAction();
      setOffset(0);
      return;
    }
    if (offset >= SWIPE_THRESHOLD && rightActions.length > 0) {
      rightActions[rightActions.length - 1].onAction();
      setOffset(0);
      return;
    }
    setOffset(0);
  }, [leftActions, offset, rightActions]);

  return (
    <div className={cn("relative overflow-hidden", className)}>
      {/* Right-edge action stack (revealed when swiping left). */}
      {leftActions.length > 0 && (
        <div className="absolute inset-y-0 right-0 flex">
          {leftActions.map((a, i) => (
            <button
              key={i}
              type="button"
              onClick={() => {
                a.onAction();
                setOffset(0);
              }}
              className={cn(
                "flex h-full w-[72px] flex-col items-center justify-center gap-0.5 text-xs font-medium text-white",
                a.color,
              )}
            >
              {a.icon}
              <span>{a.label}</span>
            </button>
          ))}
        </div>
      )}
      {/* Left-edge action stack (revealed when swiping right). */}
      {rightActions.length > 0 && (
        <div className="absolute inset-y-0 left-0 flex">
          {rightActions.map((a, i) => (
            <button
              key={i}
              type="button"
              onClick={() => {
                a.onAction();
                setOffset(0);
              }}
              className={cn(
                "flex h-full w-[72px] flex-col items-center justify-center gap-0.5 text-xs font-medium text-white",
                a.color,
              )}
            >
              {a.icon}
              <span>{a.label}</span>
            </button>
          ))}
        </div>
      )}

      <div
        className="relative bg-background touch-pan-y"
        style={{ transform: `translateX(${offset}px)`, transition: startX.current ? "none" : "transform 200ms cubic-bezier(0.25, 0.1, 0.25, 1)" }}
        onPointerDown={onPointerDown}
        onPointerMove={onPointerMove}
        onPointerUp={onPointerUp}
        onPointerCancel={onPointerUp}
      >
        {children}
      </div>
    </div>
  );
}
