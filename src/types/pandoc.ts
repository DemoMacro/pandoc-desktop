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

  // Other formats
  epub: "epub",
  org: "org",
  textile: "textile",
  wiki: "mediawiki",
  dokuwiki: "dokuwiki",
  man: "man",
  typst: "typst",
  typ: "typst",
  bib: "biblatex",
  bibtex: "bibtex",
  json: "json",
  yaml: "yaml",
  yml: "yaml",
  csv: "csv",
  tsv: "tsv",
};

// Input format to compatible output formats mapping
export const FORMAT_COMPATIBILITY: Record<string, string[]> = {
  markdown: [
    "html",
    "html5",
    "pdf",
    "latex",
    "docx",
    "odt",
    "epub",
    "epub3",
    "rst",
    "plain",
    "rtf",
    "mediawiki",
    "asciidoc",
    "org",
    "textile",
    "beamer",
    "pptx",
    "icml",
    "opendocument",
    "json",
    "native",
  ],
  html: [
    "markdown",
    "latex",
    "docx",
    "odt",
    "epub",
    "rst",
    "plain",
    "rtf",
    "mediawiki",
    "asciidoc",
    "org",
    "textile",
    "json",
    "native",
  ],
  latex: [
    "html",
    "html5",
    "pdf",
    "docx",
    "odt",
    "epub",
    "markdown",
    "rst",
    "plain",
    "rtf",
    "mediawiki",
    "asciidoc",
    "org",
    "json",
    "native",
  ],
  docx: [
    "html",
    "html5",
    "markdown",
    "latex",
    "pdf",
    "odt",
    "epub",
    "rst",
    "plain",
    "rtf",
    "mediawiki",
    "asciidoc",
    "org",
    "json",
    "native",
  ],
  odt: [
    "html",
    "html5",
    "markdown",
    "latex",
    "pdf",
    "docx",
    "epub",
    "rst",
    "plain",
    "rtf",
    "mediawiki",
    "asciidoc",
    "org",
    "json",
    "native",
  ],
  epub: [
    "html",
    "html5",
    "markdown",
    "latex",
    "docx",
    "odt",
    "rst",
    "plain",
    "rtf",
    "json",
    "native",
  ],
  rst: [
    "html",
    "html5",
    "pdf",
    "latex",
    "docx",
    "odt",
    "epub",
    "markdown",
    "plain",
    "rtf",
    "mediawiki",
    "asciidoc",
    "org",
    "json",
    "native",
  ],
  org: [
    "html",
    "html5",
    "pdf",
    "latex",
    "docx",
    "odt",
    "epub",
    "markdown",
    "rst",
    "plain",
    "rtf",
    "mediawiki",
    "asciidoc",
    "json",
    "native",
  ],
  textile: [
    "html",
    "html5",
    "pdf",
    "latex",
    "docx",
    "odt",
    "epub",
    "markdown",
    "rst",
    "plain",
    "rtf",
    "mediawiki",
    "asciidoc",
    "org",
    "json",
    "native",
  ],
  mediawiki: [
    "html",
    "html5",
    "pdf",
    "latex",
    "docx",
    "odt",
    "epub",
    "markdown",
    "rst",
    "plain",
    "rtf",
    "asciidoc",
    "org",
    "json",
    "native",
  ],
  biblatex: ["html", "markdown", "json", "native", "bibtex", "csljson"],
  bibtex: ["html", "markdown", "json", "native", "biblatex", "csljson"],
  json: ["html", "markdown", "latex", "native"],
  plain: [
    "html",
    "markdown",
    "latex",
    "docx",
    "odt",
    "epub",
    "rst",
    "rtf",
    "json",
    "native",
  ],
  rtf: [
    "html",
    "markdown",
    "latex",
    "docx",
    "odt",
    "rst",
    "plain",
    "json",
    "native",
  ],
  man: ["html", "markdown", "latex", "plain", "rst", "json", "native"],
  dokuwiki: [
    "html",
    "markdown",
    "latex",
    "docx",
    "odt",
    "rst",
    "plain",
    "json",
    "native",
  ],
  typst: [
    "html",
    "markdown",
    "latex",
    "pdf",
    "docx",
    "odt",
    "rst",
    "plain",
    "json",
    "native",
  ],
  native: [
    "html",
    "html5",
    "pdf",
    "latex",
    "docx",
    "odt",
    "epub",
    "markdown",
    "rst",
    "plain",
    "rtf",
    "mediawiki",
    "asciidoc",
    "org",
    "textile",
    "beamer",
    "pptx",
    "icml",
    "opendocument",
    "json",
  ],
};

// Output format definitions with human-readable labels and file extensions
export interface OutputFormat {
  value: string;
  label: string;
  ext: string;
  description?: string;
}

export const OUTPUT_FORMATS: OutputFormat[] = [
  {
    value: "html",
    label: "HTML",
    ext: "html",
    description: "HyperText Markup Language",
  },
  { value: "html5", label: "HTML5", ext: "html", description: "HTML5 format" },
  {
    value: "pdf",
    label: "PDF",
    ext: "pdf",
    description: "Portable Document Format",
  },
  {
    value: "latex",
    label: "LaTeX",
    ext: "tex",
    description: "LaTeX typesetting",
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
    value: "epub",
    label: "EPUB",
    ext: "epub",
    description: "Electronic Publication",
  },
  {
    value: "epub3",
    label: "EPUB3",
    ext: "epub",
    description: "EPUB version 3",
  },
  {
    value: "markdown",
    label: "Markdown",
    ext: "md",
    description: "Markdown format",
  },
  {
    value: "rst",
    label: "reStructuredText",
    ext: "rst",
    description: "reStructuredText",
  },
  {
    value: "plain",
    label: "Plain Text",
    ext: "txt",
    description: "Plain text format",
  },
  {
    value: "rtf",
    label: "Rich Text Format",
    ext: "rtf",
    description: "Rich Text Format",
  },
  {
    value: "mediawiki",
    label: "MediaWiki",
    ext: "wiki",
    description: "MediaWiki markup",
  },
  {
    value: "asciidoc",
    label: "AsciiDoc",
    ext: "adoc",
    description: "AsciiDoc format",
  },
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
    value: "beamer",
    label: "Beamer Slides",
    ext: "tex",
    description: "LaTeX Beamer presentation",
  },
  {
    value: "pptx",
    label: "PowerPoint",
    ext: "pptx",
    description: "Microsoft PowerPoint",
  },
  {
    value: "icml",
    label: "InDesign ICML",
    ext: "icml",
    description: "Adobe InDesign",
  },
  {
    value: "opendocument",
    label: "OpenDocument",
    ext: "odt",
    description: "OpenDocument format",
  },
  { value: "json", label: "JSON", ext: "json", description: "JSON format" },
  {
    value: "native",
    label: "Pandoc Native",
    ext: "native",
    description: "Pandoc AST",
  },
  { value: "ms", label: "Groff MS", ext: "ms", description: "Groff MS format" },
  {
    value: "man",
    label: "Man Page",
    ext: "man",
    description: "Unix manual page",
  },
  {
    value: "dokuwiki",
    label: "DokuWiki",
    ext: "dokuwiki",
    description: "DokuWiki markup",
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

// Utility functions
export function getFileExtension(filepath: string): string {
  const lastDot = filepath.lastIndexOf(".");
  if (lastDot === -1 || lastDot === filepath.length - 1) {
    return "";
  }
  return filepath.substring(lastDot + 1).toLowerCase();
}

// Get filename without extension from path
export function getBaseName(filepath: string): string {
  // First get the filename from path
  const filename = filepath.split(/[/\\]/).pop() || "";
  // Then remove extension
  const lastDot = filename.lastIndexOf(".");
  if (lastDot === -1 || lastDot === filename.length - 1) {
    return filename;
  }
  return filename.substring(0, lastDot);
}

export function detectInputFormat(filepath: string): string {
  const ext = getFileExtension(filepath);
  return INPUT_FORMAT_MAP[ext] || "markdown";
}

export function getOutputFormatByValue(
  value: string,
): OutputFormat | undefined {
  return OUTPUT_FORMATS.find((format) => format.value === value);
}

export function generateOutputFilename(
  inputPath: string,
  outputFormat: string,
): string {
  const baseName = inputPath.replace(/\.[^/.]+$/, "");
  const outputFormatObj = getOutputFormatByValue(outputFormat);
  const ext = outputFormatObj?.ext || outputFormat;
  return `${baseName}.${ext}`;
}

// Generate output filename with extension for display
export function generateOutputFilenameWithExt(
  baseName: string,
  outputFormat: string,
): string {
  const outputFormatObj = getOutputFormatByValue(outputFormat);
  const ext = outputFormatObj?.ext || outputFormat;
  return `${baseName}.${ext}`;
}

export function getCompatibleOutputFormats(
  inputFormat: string,
  supportedOutputs: string[],
): OutputFormat[] {
  // Get compatible formats for this input format
  const compatibleFormats = FORMAT_COMPATIBILITY[inputFormat] || [];

  return OUTPUT_FORMATS.filter((format) => {
    // Check if format is supported by current Pandoc installation
    if (!supportedOutputs.includes(format.value)) {
      return false;
    }

    // Check if this input format can convert to this output format
    if (
      compatibleFormats.length > 0 &&
      !compatibleFormats.includes(format.value)
    ) {
      return false;
    }

    return true;
  });
}
