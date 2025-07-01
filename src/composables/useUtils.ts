// Import types and constants from pandoc.ts
import type { OutputFormat } from "../types/pandoc";
import { INPUT_FORMAT_MAP, OUTPUT_FORMATS } from "../types/pandoc";

/**
 * Format bytes to human readable format
 */
export const formatBytes = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + " " + sizes[i];
};

/**
 * Extract filename from full path
 */
export const getFileName = (path: string): string => {
  return path.split(/[/\\]/).pop() || "";
};

/**
 * Get short path for display (show only last 2 segments)
 */
export const getShortPath = (path: string): string => {
  const parts = path.split(/[/\\]/);
  if (parts.length <= 3) return path;
  return `.../${parts.slice(-2).join("/")}`;
};

/**
 * Extract version number from version string
 */
export const extractVersionNumber = (versionStr: string): string => {
  const versionMatch = versionStr.match(/(\d+\.\d+\.\d+(?:\.\d+)?)/);
  return versionMatch ? versionMatch[1] : versionStr.trim();
};

/**
 * Clean version string for comparison (remove 'v' prefix)
 */
export const cleanVersionString = (version: string): string => {
  return version.replace(/^v/, "").trim();
};

/**
 * Clean Windows path (remove long path prefix)
 */
export const cleanWindowsPath = (path: string): string => {
  // Remove Windows long path prefix \\?\
  return path.replace(/^\\\\\?\\/g, "");
};

// Get file extension from path
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

// Detect input format from file extension
export function detectInputFormat(filepath: string): string {
  const ext = getFileExtension(filepath);
  return INPUT_FORMAT_MAP[ext] || "markdown";
}

// Get output format object by value
export function getOutputFormatByValue(
  value: string,
): OutputFormat | undefined {
  return OUTPUT_FORMATS.find((format) => format.value === value);
}

// Generate output filename from input path and format
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

// Get supported output formats
export function getSupportedOutputFormats(
  supportedOutputs: string[],
): OutputFormat[] {
  // Filter formats by what the current Pandoc installation actually supports
  return OUTPUT_FORMATS.filter((format) =>
    supportedOutputs.includes(format.value),
  );
}
