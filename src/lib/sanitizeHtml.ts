/**
 * Minimal HTML sanitizer for V1.
 * Strips <script>, <style>, and on* attributes to prevent XSS.
 *
 * TODO: Replace with a proper sanitization library (e.g. DOMPurify)
 * when full HTML email rendering is needed in V2.
 */
export function stripDangerousTags(html: string): string {
  return html
    // Remove script blocks (including content)
    .replace(/<script\b[^<]*(?:(?!<\/script>)<[^<]*)*<\/script>/gi, "")
    // Remove style blocks
    .replace(/<style\b[^<]*(?:(?!<\/style>)<[^<]*)*<\/style>/gi, "")
    // Remove on* event handler attributes
    .replace(/\s+on\w+="[^"]*"/gi, "")
    .replace(/\s+on\w+='[^']*'/gi, "")
    // Remove javascript: hrefs
    .replace(/href\s*=\s*["']?\s*javascript:[^"'\s>]*/gi, 'href="#"');
}
