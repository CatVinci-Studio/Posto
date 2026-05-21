import { useEffect, useState } from "react";

/**
 * Subscribe to a CSS media query. Returns true when the query currently matches.
 *
 * Useful for routing layout choices off Tailwind breakpoints in JS (e.g. swap
 * three-pane vs single-pane when the window narrows below `md`).
 */
export function useMediaQuery(query: string): boolean {
  const [matches, setMatches] = useState(() => {
    if (typeof window === "undefined") return false;
    return window.matchMedia(query).matches;
  });

  useEffect(() => {
    if (typeof window === "undefined") return;
    const mql = window.matchMedia(query);
    const onChange = (e: MediaQueryListEvent) => setMatches(e.matches);
    mql.addEventListener("change", onChange);
    setMatches(mql.matches);
    return () => mql.removeEventListener("change", onChange);
  }, [query]);

  return matches;
}

/** Convenience combined hook: classify the viewport into a coarse bucket. */
export function useScreenSize() {
  const isMobile = useMediaQuery("(max-width: 767px)");
  const isTablet = useMediaQuery("(min-width: 768px) and (max-width: 1023px)");
  const isDesktop = useMediaQuery("(min-width: 1024px)");
  const isWide = useMediaQuery("(min-width: 1280px)");
  return { isMobile, isTablet, isDesktop, isWide };
}
