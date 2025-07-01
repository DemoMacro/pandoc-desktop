// Pandoc information structure
export interface PandocInfo {
  version: string;
  path: string;
  is_working: boolean;
  supported_input_formats: string[];
  supported_output_formats: string[];
  detected_paths: string[];
  search_paths: string[];
}

// Pandoc source types for new manager system
export interface PandocSource {
  Custom?: string;
  Bundled?: null;
  System?: string;
}

// Pandoc manager structure for new unified system
export interface PandocManager {
  source: PandocSource;
  info: PandocInfo | null;
  available: boolean;
}

// Version management types
export interface PandocRelease {
  tag_name: string;
  name: string;
  body: string;
  published_at: string;
  assets: Array<{
    name: string;
    download_url: string;
    size: number;
    content_type: string;
  }>;
}

export interface VersionInfo {
  current: string | null;
  latest: string | null;
  available_versions: string[];
  is_update_available: boolean;
}

export interface DownloadProgress {
  downloaded: number;
  total: number;
  percentage: number;
  speed: string;
}

// Pandoc configuration
export interface PandocConfig {
  custom_path?: string;
  use_custom_path: boolean;
  last_detected_version: string;
}

// Format mapping for file extensions to pandoc input formats
export const INPUT_FORMAT_MAP: Record<string, string> = {
  // Markdown formats
  md: "markdown",
  markdown: "markdown",
  mdown: "markdown",
  mkd: "markdown",
  mkdn: "markdown",

  // HTML formats
  html: "html",
  htm: "html",

  // Document formats
  docx: "docx",
  odt: "odt",
  rtf: "rtf",

  // LaTeX formats
  tex: "latex",
  latex: "latex",

  // reStructuredText
  rst: "rst",
  rest: "rst",

  // Plain text
  txt: "plain",
  text: "plain",

  // Bibliography formats
  bib: "biblatex",
  biblatex: "biblatex",
  bibtex: "bibtex",

  // Data formats
  json: "json",
  yaml: "yaml",
  yml: "yaml",
  csv: "csv",
  tsv: "tsv",

  // Document formats
  epub: "epub",
  org: "org",
  textile: "textile",
  man: "man",
  typst: "typst",
  typ: "typst",

  // Wiki formats
  wiki: "mediawiki",
  mediawiki: "mediawiki",
  dokuwiki: "dokuwiki",
  tikiwiki: "tikiwiki",
  twiki: "twiki",
  vimwiki: "vimwiki",

  // XML-based formats
  xml: "docbook",
  docbook: "docbook",

  // Other formats
  fb2: "fb2",
  opml: "opml",
  t2t: "t2t",
  ipynb: "ipynb",
  muse: "muse",
  ris: "ris",
  jats: "jats",
  jira: "jira",
  creole: "creole",
  mdoc: "mdoc",
  pod: "pod",
  endnotexml: "endnotexml",
  bits: "bits",
  djot: "djot",
  csljson: "csljson",

  // GitHub flavored markdown
  gfm: "gfm",

  // CommonMark
  commonmark: "commonmark",

  // Haddock
  haddock: "haddock",

  // Native pandoc format
  native: "native",
};

// Output format definitions with human-readable labels and file extensions
export interface OutputFormat {
  value: string;
  label: string;
  ext: string;
  description?: string;
}

export const OUTPUT_FORMATS: OutputFormat[] = [
  // HTML formats
  {
    value: "html",
    label: "HTML",
    ext: "html",
    description: "HyperText Markup Language",
  },
  { value: "html4", label: "HTML4", ext: "html", description: "HTML4 format" },
  { value: "html5", label: "HTML5", ext: "html", description: "HTML5 format" },
  {
    value: "chunkedhtml",
    label: "Chunked HTML",
    ext: "html",
    description: "Chunked HTML",
  },

  // Document formats
  {
    value: "pdf",
    label: "PDF",
    ext: "pdf",
    description: "Portable Document Format",
  },
  {
    value: "docx",
    label: "Word Document",
    ext: "docx",
    description: "Microsoft Word",
  },
  {
    value: "odt",
    label: "OpenDocument Text",
    ext: "odt",
    description: "LibreOffice Writer",
  },
  {
    value: "opendocument",
    label: "OpenDocument",
    ext: "odt",
    description: "OpenDocument format",
  },

  // LaTeX formats
  {
    value: "latex",
    label: "LaTeX",
    ext: "tex",
    description: "LaTeX typesetting",
  },
  {
    value: "beamer",
    label: "Beamer Slides",
    ext: "tex",
    description: "LaTeX Beamer presentation",
  },
  {
    value: "context",
    label: "ConTeXt",
    ext: "tex",
    description: "ConTeXt format",
  },

  // EPUB formats
  {
    value: "epub",
    label: "EPUB",
    ext: "epub",
    description: "Electronic Publication",
  },
  {
    value: "epub2",
    label: "EPUB2",
    ext: "epub",
    description: "EPUB version 2",
  },
  {
    value: "epub3",
    label: "EPUB3",
    ext: "epub",
    description: "EPUB version 3",
  },

  // Markdown variants
  {
    value: "markdown",
    label: "Markdown",
    ext: "md",
    description: "Markdown format",
  },
  {
    value: "commonmark",
    label: "CommonMark",
    ext: "md",
    description: "CommonMark standard",
  },
  {
    value: "commonmark_x",
    label: "CommonMark-X",
    ext: "md",
    description: "CommonMark extensions",
  },
  {
    value: "gfm",
    label: "GitHub Markdown",
    ext: "md",
    description: "GitHub Flavored Markdown",
  },
  {
    value: "markdown_github",
    label: "Markdown (GitHub)",
    ext: "md",
    description: "GitHub style Markdown",
  },
  {
    value: "markdown_mmd",
    label: "MultiMarkdown",
    ext: "md",
    description: "MultiMarkdown",
  },
  {
    value: "markdown_phpextra",
    label: "Markdown (PHP Extra)",
    ext: "md",
    description: "PHP Markdown Extra",
  },
  {
    value: "markdown_strict",
    label: "Markdown (Strict)",
    ext: "md",
    description: "Strict Markdown",
  },
  {
    value: "markua",
    label: "Markua",
    ext: "markua",
    description: "Markua format",
  },

  // Text formats
  {
    value: "plain",
    label: "Plain Text",
    ext: "txt",
    description: "Plain text format",
  },
  {
    value: "rst",
    label: "reStructuredText",
    ext: "rst",
    description: "reStructuredText",
  },
  {
    value: "rtf",
    label: "Rich Text Format",
    ext: "rtf",
    description: "Rich Text Format",
  },
  {
    value: "ansi",
    label: "ANSI",
    ext: "txt",
    description: "ANSI colored text",
  },

  // Wiki formats
  {
    value: "mediawiki",
    label: "MediaWiki",
    ext: "wiki",
    description: "MediaWiki markup",
  },
  {
    value: "dokuwiki",
    label: "DokuWiki",
    ext: "dokuwiki",
    description: "DokuWiki markup",
  },
  {
    value: "xwiki",
    label: "XWiki",
    ext: "xwiki",
    description: "XWiki markup",
  },
  {
    value: "zimwiki",
    label: "ZimWiki",
    ext: "zimwiki",
    description: "Zim Wiki markup",
  },

  // AsciiDoc formats
  {
    value: "asciidoc",
    label: "AsciiDoc",
    ext: "adoc",
    description: "AsciiDoc format",
  },
  {
    value: "asciidoc_legacy",
    label: "AsciiDoc (Legacy)",
    ext: "adoc",
    description: "Legacy AsciiDoc",
  },
  {
    value: "asciidoctor",
    label: "Asciidoctor",
    ext: "adoc",
    description: "Asciidoctor format",
  },

  // Other markup
  {
    value: "org",
    label: "Org Mode",
    ext: "org",
    description: "Emacs Org mode",
  },
  {
    value: "textile",
    label: "Textile",
    ext: "textile",
    description: "Textile markup",
  },
  {
    value: "haddock",
    label: "Haddock",
    ext: "haddock",
    description: "Haskell documentation",
  },
  {
    value: "typst",
    label: "Typst",
    ext: "typ",
    description: "Typst typesetting",
  },
  { value: "muse", label: "Muse", ext: "muse", description: "Emacs Muse" },

  // Presentation formats
  {
    value: "pptx",
    label: "PowerPoint",
    ext: "pptx",
    description: "Microsoft PowerPoint",
  },
  {
    value: "revealjs",
    label: "reveal.js",
    ext: "html",
    description: "reveal.js presentation",
  },
  {
    value: "dzslides",
    label: "DZSlides",
    ext: "html",
    description: "DZSlides presentation",
  },
  { value: "s5", label: "S5", ext: "html", description: "S5 slideshow" },
  {
    value: "slideous",
    label: "Slideous",
    ext: "html",
    description: "Slideous slideshow",
  },
  {
    value: "slidy",
    label: "Slidy",
    ext: "html",
    description: "Slidy slideshow",
  },

  // Publishing formats
  {
    value: "icml",
    label: "InDesign ICML",
    ext: "icml",
    description: "Adobe InDesign",
  },
  {
    value: "tei",
    label: "TEI",
    ext: "xml",
    description: "Text Encoding Initiative",
  },

  // Data formats
  { value: "json", label: "JSON", ext: "json", description: "JSON format" },
  {
    value: "csljson",
    label: "CSL JSON",
    ext: "json",
    description: "Citation Style Language JSON",
  },
  {
    value: "native",
    label: "Pandoc Native",
    ext: "native",
    description: "Pandoc AST",
  },

  // Bibliography formats
  {
    value: "biblatex",
    label: "BibLaTeX",
    ext: "bib",
    description: "BibLaTeX bibliography",
  },
  {
    value: "bibtex",
    label: "BibTeX",
    ext: "bib",
    description: "BibTeX bibliography",
  },

  // System formats
  { value: "ms", label: "Groff MS", ext: "ms", description: "Groff MS format" },
  {
    value: "man",
    label: "Man Page",
    ext: "man",
    description: "Unix manual page",
  },
  {
    value: "texinfo",
    label: "Texinfo",
    ext: "texi",
    description: "GNU Texinfo",
  },

  // Special formats
  {
    value: "fb2",
    label: "FictionBook2",
    ext: "fb2",
    description: "FictionBook2 e-book",
  },
  {
    value: "ipynb",
    label: "Jupyter Notebook",
    ext: "ipynb",
    description: "Jupyter Notebook",
  },
  {
    value: "opml",
    label: "OPML",
    ext: "opml",
    description: "Outline Processor Markup Language",
  },
  { value: "djot", label: "Djot", ext: "djot", description: "Djot markup" },

  // XML/DocBook formats
  {
    value: "docbook",
    label: "DocBook",
    ext: "xml",
    description: "DocBook XML",
  },
  {
    value: "docbook4",
    label: "DocBook 4",
    ext: "xml",
    description: "DocBook 4.x",
  },
  {
    value: "docbook5",
    label: "DocBook 5",
    ext: "xml",
    description: "DocBook 5.x",
  },

  // JATS formats
  {
    value: "jats",
    label: "JATS",
    ext: "xml",
    description: "Journal Article Tag Suite",
  },
  {
    value: "jats_archiving",
    label: "JATS Archiving",
    ext: "xml",
    description: "JATS Archiving DTD",
  },
  {
    value: "jats_articleauthoring",
    label: "JATS Article Authoring",
    ext: "xml",
    description: "JATS Article Authoring DTD",
  },
  {
    value: "jats_publishing",
    label: "JATS Publishing",
    ext: "xml",
    description: "JATS Publishing DTD",
  },

  // Other formats
  {
    value: "jira",
    label: "Jira Wiki",
    ext: "jira",
    description: "Jira wiki markup",
  },
];

// Error types for better error handling
export enum PandocError {
  NOT_FOUND = "pandoc_not_found",
  INVALID_PATH = "invalid_path",
  VERSION_TOO_OLD = "version_too_old",
  CONVERSION_FAILED = "conversion_failed",
  FORMAT_NOT_SUPPORTED = "format_not_supported",
  FILE_NOT_FOUND = "file_not_found",
  PERMISSION_DENIED = "permission_denied",
}

export interface ErrorInfo {
  title: string;
  message: string;
  actions: string[];
}

export const ERROR_MESSAGES: Record<PandocError, ErrorInfo> = {
  [PandocError.NOT_FOUND]: {
    title: "Pandoc Not Found",
    message: "Pandoc is not installed or not found in system PATH",
    actions: ["Install Pandoc", "Set Custom Path", "View Installation Guide"],
  },
  [PandocError.INVALID_PATH]: {
    title: "Invalid Path",
    message: "The specified Pandoc path is invalid or file does not exist",
    actions: ["Select Different Path", "Use Auto Detection"],
  },
  [PandocError.VERSION_TOO_OLD]: {
    title: "Version Too Old",
    message:
      "Current Pandoc version is too old and may not support some features",
    actions: ["Update Pandoc", "Download Latest Version"],
  },
  [PandocError.CONVERSION_FAILED]: {
    title: "Conversion Failed",
    message: "An error occurred during file conversion",
    actions: ["Check Input File", "Try Different Format", "View Error Details"],
  },
  [PandocError.FORMAT_NOT_SUPPORTED]: {
    title: "Format Not Supported",
    message: "Current Pandoc version does not support this format",
    actions: [
      "Choose Different Format",
      "Update Pandoc",
      "View Supported Formats",
    ],
  },
  [PandocError.FILE_NOT_FOUND]: {
    title: "File Not Found",
    message: "The specified input file does not exist",
    actions: ["Check File Path", "Select Different File"],
  },
  [PandocError.PERMISSION_DENIED]: {
    title: "Permission Denied",
    message: "No permission to read input file or write output file",
    actions: ["Check File Permissions", "Choose Different Location"],
  },
};

// Note: Utility functions have been moved to src/composables/useUtils.ts
