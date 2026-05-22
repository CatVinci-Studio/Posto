import DOMPurify from "isomorphic-dompurify";

const ALLOWED_TAGS = [
  "a", "abbr", "address", "article", "aside", "b", "bdi", "bdo", "blockquote",
  "br", "caption", "center", "cite", "code", "col", "colgroup", "data", "dd",
  "del", "details", "dfn", "div", "dl", "dt", "em", "figcaption", "figure",
  "footer", "h1", "h2", "h3", "h4", "h5", "h6", "header", "hgroup", "hr", "i",
  "img", "ins", "kbd", "li", "main", "mark", "meter", "nav", "ol", "p", "pre",
  "q", "rb", "rp", "rt", "rtc", "ruby", "s", "samp", "section", "small",
  "span", "strong", "sub", "summary", "sup", "table", "tbody", "td", "tfoot",
  "th", "thead", "time", "tr", "u", "ul", "var", "wbr",
];

const ALLOWED_ATTR = [
  "href", "title", "alt", "src", "width", "height", "align", "valign",
  "cellpadding", "cellspacing", "border", "colspan", "rowspan", "style",
  "class", "name", "color", "bgcolor",
];

const RAW_OPTIONS = {
  ALLOWED_TAGS,
  ALLOWED_ATTR,
  ALLOW_DATA_ATTR: false,
  FORBID_TAGS: ["script", "style", "iframe", "object", "embed", "link", "meta", "form"],
  FORBID_ATTR: ["onerror", "onload", "onclick", "onmouseover", "onfocus", "onblur"],
  ALLOWED_URI_REGEXP:
    /^(?:(?:https?|mailto|tel|cid|ftp):|[^a-z]|[a-z+.-]+(?:[^a-z+.:-]|$))/i,
  RETURN_TRUSTED_TYPE: false,
};

export function sanitizeHtml(html: string): string {
  return DOMPurify.sanitize(html, RAW_OPTIONS) as unknown as string;
}

/** @deprecated Use sanitizeHtml — kept for callers that still import it. */
export const stripDangerousTags = sanitizeHtml;
