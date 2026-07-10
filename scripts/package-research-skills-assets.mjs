#!/usr/bin/env node
import { createHash } from 'node:crypto';
import { readdir, readFile, stat, writeFile, mkdir } from 'node:fs/promises';
import path from 'node:path';
import { gzipSync } from 'node:zlib';

const ASSET = 'research-skills';
const SCHEMA_VERSION = 1;
const ROOTS = [
  'skills/deep-research.md',
  'skills/pm-deep-research.md',
  'skills/academic-deep-research.md',
  'skills/technical-evaluation.md',
  'prompts/layer1/task-decomposition.md',
  'prompts/layer1/final-report.md',
  'prompts/layer2/aspect-agent.md',
  'prompts/layer2/search-planner.md',
  'prompts/layer2/evidence-extractor.md',
  'prompts/layer1/common',
  'prompts/layer1/pm-deep-research',
  'prompts/layer2/pm-deep-research',
  'prompts/layer1/academic-deep-research',
  'prompts/layer2/academic-deep-research',
  'prompts/layer1/technical-evaluation',
  'prompts/layer2/technical-evaluation',
];

const ALLOWED_FILES = new Set([
  'skills/deep-research.md',
  'skills/pm-deep-research.md',
  'skills/academic-deep-research.md',
  'skills/technical-evaluation.md',
  'prompts/layer1/task-decomposition.md',
  'prompts/layer1/final-report.md',
  'prompts/layer2/aspect-agent.md',
  'prompts/layer2/search-planner.md',
  'prompts/layer2/evidence-extractor.md',
]);

const ALLOWED_PREFIXES = [
  'prompts/layer1/common/',
  'prompts/layer1/pm-deep-research/',
  'prompts/layer2/pm-deep-research/',
  'prompts/layer1/academic-deep-research/',
  'prompts/layer2/academic-deep-research/',
  'prompts/layer1/technical-evaluation/',
  'prompts/layer2/technical-evaluation/',
];

const args = parseArgs(process.argv.slice(2));
const version = args.version ?? process.env.VERSION;
const commit = args.commit ?? process.env.GITHUB_SHA ?? null;
const outputDir = args.outputDir ?? 'artifacts';

if (!version) {
  throw new Error('missing --version');
}

await mkdir(outputDir, { recursive: true });

const files = await collectAssetFiles(process.cwd());
const archiveName = `${ASSET}-assets-v${version}.tar.gz`;
const manifestName = `${ASSET}-assets-v${version}.manifest.json`;
const archive = gzipSync(buildTar(files), { level: 9 });

await writeFile(path.join(outputDir, archiveName), archive);
await writeFile(
  path.join(outputDir, manifestName),
  `${JSON.stringify(
    {
      schema_version: SCHEMA_VERSION,
      asset: ASSET,
      version,
      archive: archiveName,
      sha256: sha256(archive),
      source_commit: commit,
      files: files.map((file) => ({
        path: file.relativePath,
        sha256: sha256(file.bytes),
        size: file.bytes.length,
      })),
    },
    null,
    2,
  )}\n`,
);

function parseArgs(argv) {
  const parsed = {};
  for (let index = 0; index < argv.length; index += 1) {
    const arg = argv[index];
    if (arg === '--version') {
      parsed.version = requireValue(argv, ++index, arg);
    } else if (arg === '--commit') {
      parsed.commit = requireValue(argv, ++index, arg);
    } else if (arg === '--output-dir') {
      parsed.outputDir = requireValue(argv, ++index, arg);
    } else {
      throw new Error(`unknown argument: ${arg}`);
    }
  }
  return parsed;
}

function requireValue(argv, index, flag) {
  const value = argv[index];
  if (!value || value.startsWith('--')) {
    throw new Error(`${flag} requires a value`);
  }
  return value;
}

async function collectAssetFiles(repoRoot) {
  const files = [];
  for (const root of ROOTS) {
    const absolute = path.join(repoRoot, root);
    const metadata = await stat(absolute);
    if (metadata.isFile()) {
      files.push(await readAssetFile(repoRoot, absolute));
    } else if (metadata.isDirectory()) {
      files.push(...await readAssetDirectory(repoRoot, absolute));
    } else {
      throw new Error(`unsupported asset path: ${root}`);
    }
  }
  files.sort((left, right) => left.relativePath.localeCompare(right.relativePath));
  validateFiles(files);
  return files;
}

async function readAssetDirectory(repoRoot, directory) {
  const entries = await readdir(directory, { withFileTypes: true });
  const files = [];
  for (const entry of entries) {
    const absolute = path.join(directory, entry.name);
    if (entry.isDirectory()) {
      files.push(...await readAssetDirectory(repoRoot, absolute));
    } else if (entry.isFile()) {
      files.push(await readAssetFile(repoRoot, absolute));
    } else {
      throw new Error(`unsupported asset entry: ${absolute}`);
    }
  }
  return files;
}

async function readAssetFile(repoRoot, absolute) {
  const relativePath = path.relative(repoRoot, absolute).split(path.sep).join('/');
  return {
    relativePath,
    bytes: await readFile(absolute),
  };
}

function validateFiles(files) {
  const seen = new Set();
  for (const file of files) {
    validateAssetPath(file.relativePath);
    if (seen.has(file.relativePath)) {
      throw new Error(`duplicate asset file: ${file.relativePath}`);
    }
    seen.add(file.relativePath);
  }
}

function validateAssetPath(relativePath) {
  const isAllowed = ALLOWED_FILES.has(relativePath)
    || ALLOWED_PREFIXES.some((prefix) => relativePath.startsWith(prefix));
  if (!isAllowed) {
    throw new Error(`unexpected asset path: ${relativePath}`);
  }
  const components = relativePath.split('/');
  if (
    relativePath.startsWith('/')
    || relativePath.includes('\\')
    || components.includes('..')
    || components.includes('.')
  ) {
    throw new Error(`unsafe asset path: ${relativePath}`);
  }
}

function buildTar(files) {
  const chunks = [];
  for (const file of files) {
    chunks.push(tarHeader(file.relativePath, file.bytes.length));
    chunks.push(file.bytes);
    chunks.push(Buffer.alloc(paddingLength(file.bytes.length)));
  }
  chunks.push(Buffer.alloc(1024));
  return Buffer.concat(chunks);
}

function tarHeader(name, size) {
  const header = Buffer.alloc(512, 0);
  writeString(header, name, 0, 100);
  writeOctal(header, 0o644, 100, 8);
  writeOctal(header, 0, 108, 8);
  writeOctal(header, 0, 116, 8);
  writeOctal(header, size, 124, 12);
  writeOctal(header, 0, 136, 12);
  header.fill(0x20, 148, 156);
  header[156] = '0'.charCodeAt(0);
  writeString(header, 'ustar', 257, 6);
  writeString(header, '00', 263, 2);

  let checksum = 0;
  for (const byte of header) {
    checksum += byte;
  }
  writeOctal(header, checksum, 148, 8);
  return header;
}

function writeString(buffer, value, offset, length) {
  const bytes = Buffer.from(value);
  if (bytes.length > length) {
    throw new Error(`tar path too long: ${value}`);
  }
  bytes.copy(buffer, offset);
}

function writeOctal(buffer, value, offset, length) {
  const text = value.toString(8).padStart(length - 1, '0');
  if (text.length > length - 1) {
    throw new Error(`tar octal value too large: ${value}`);
  }
  buffer.write(text, offset, length - 1, 'ascii');
  buffer[offset + length - 1] = 0;
}

function paddingLength(size) {
  const remainder = size % 512;
  return remainder === 0 ? 0 : 512 - remainder;
}

function sha256(bytes) {
  return createHash('sha256').update(bytes).digest('hex');
}
