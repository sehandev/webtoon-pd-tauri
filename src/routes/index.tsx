"use client";

import {
  ArrowClockwise,
  FileArrowUp,
  FloppyDisk,
  FolderOpen,
  TextB,
  TextItalic,
} from "@phosphor-icons/react";
import { createFileRoute } from "@tanstack/react-router";
import { invoke } from "@tauri-apps/api/core";
import { useMemo, useRef, useState } from "react";
import { Button } from "@/components/ui/button";

export const Route = createFileRoute("/")({ component: App });

type SourceFormat = "clip" | "psd" | "psb" | "jpg" | "png" | "webp" | "avif";

type SourceFileSummary = {
  path: string;
  fileName: string;
  format: SourceFormat;
  sizeBytes: number;
  sha256: string;
};

type BackendConfig = {
  workerBaseUrl: string;
  supabaseUrl: string;
  supabaseAnonKey: string;
  supabaseAccessToken: string;
};

type WorkspaceContext = {
  organizationId: string;
  projectId: string;
  episodeId: string;
  stageId: string;
  createdBy: string;
};

type ToolConfig = {
  rendererBin: string;
  avifencBin: string;
};

type ProcessSourceAssetResponse = {
  assetId: string;
  reviewAssetId: string;
  assetProcessingJobId: string;
  originalKey: string;
  tileCount: number;
  renderedWidth: number;
  renderedHeight: number;
};

const initialBackend: BackendConfig = {
  workerBaseUrl: "",
  supabaseUrl: "",
  supabaseAnonKey: "",
  supabaseAccessToken: "",
};

const initialWorkspace: WorkspaceContext = {
  organizationId: "",
  projectId: "",
  episodeId: "",
  stageId: "",
  createdBy: "",
};

const initialTools: ToolConfig = {
  rendererBin: "",
  avifencBin: "",
};

function App() {
  const [backend, setBackend] = useState(initialBackend);
  const [workspace, setWorkspace] = useState(initialWorkspace);
  const [tools, setTools] = useState(initialTools);
  const [sourceFile, setSourceFile] = useState<SourceFileSummary | null>(null);
  const [processing, setProcessing] = useState(false);
  const [savingDocument, setSavingDocument] = useState(false);
  const [loadingDocument, setLoadingDocument] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [result, setResult] = useState<ProcessSourceAssetResponse | null>(null);
  const editorRef = useRef<HTMLDivElement>(null);

  const canProcess = useMemo(
    () =>
      Boolean(
        sourceFile &&
          backend.workerBaseUrl &&
          backend.supabaseUrl &&
          backend.supabaseAnonKey &&
          backend.supabaseAccessToken &&
          workspace.organizationId &&
          workspace.projectId &&
          workspace.episodeId &&
          workspace.stageId &&
          workspace.createdBy,
      ),
    [backend, sourceFile, workspace],
  );

  async function selectSourceFile() {
    setError(null);
    setResult(null);
    const selected = await invoke<SourceFileSummary | null>(
      "select_source_file",
    );
    setSourceFile(selected);
  }

  async function processSelectedSource() {
    if (!sourceFile) {
      return;
    }
    setProcessing(true);
    setError(null);
    setResult(null);
    try {
      const response = await invoke<ProcessSourceAssetResponse>(
        "process_source_asset",
        {
          request: {
            sourcePath: sourceFile.path,
            backend,
            workspace,
            tools: emptyStringsToUndefined(tools),
          },
        },
      );
      setResult(response);
    } catch (caught) {
      setError(String(caught));
    } finally {
      setProcessing(false);
    }
  }

  async function saveDocument() {
    const editor = editorRef.current;
    if (!editor) {
      return;
    }
    setSavingDocument(true);
    setError(null);
    try {
      await invoke("save_wysiwyg_document", {
        request: {
          backend,
          workspace,
          contentHtml: editor.innerHTML,
          contentText: editor.innerText,
        },
      });
    } catch (caught) {
      setError(String(caught));
    } finally {
      setSavingDocument(false);
    }
  }

  async function loadDocument() {
    setLoadingDocument(true);
    setError(null);
    try {
      const document = await invoke<{ contentHtml: string } | null>(
        "load_wysiwyg_document",
        {
          backend,
          stageId: workspace.stageId,
        },
      );
      if (editorRef.current) {
        editorRef.current.innerHTML = document?.contentHtml ?? "";
      }
    } catch (caught) {
      setError(String(caught));
    } finally {
      setLoadingDocument(false);
    }
  }

  function formatDocument(command: "bold" | "italic") {
    document.execCommand(command);
    editorRef.current?.focus();
  }

  return (
    <main className="grid h-svh grid-cols-[300px_minmax(0,1fr)] overflow-hidden bg-background text-foreground">
      <aside className="border-border overflow-y-auto border-r bg-muted/30 p-4">
        <div className="flex flex-col gap-5">
          <section className="flex flex-col gap-3">
            <div>
              <h1 className="text-sm font-semibold">Webtoon PD MVP</h1>
              <p className="text-muted-foreground text-xs">
                Tauri Rust workflow
              </p>
            </div>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <StatusPill label="Tile width" value="2048px" />
              <StatusPill label="Tile height" value="2048px" />
              <StatusPill label="AVIF" value="lossless" />
              <StatusPill label="Chroma" value="4:4:4" />
            </div>
          </section>

          <section className="flex flex-col gap-2">
            <SectionTitle>Backend</SectionTitle>
            <TextInput
              label="Worker URL"
              value={backend.workerBaseUrl}
              onChange={(workerBaseUrl) =>
                setBackend((current) => ({ ...current, workerBaseUrl }))
              }
            />
            <TextInput
              label="Supabase URL"
              value={backend.supabaseUrl}
              onChange={(supabaseUrl) =>
                setBackend((current) => ({ ...current, supabaseUrl }))
              }
            />
            <TextInput
              label="Supabase anon key"
              value={backend.supabaseAnonKey}
              onChange={(supabaseAnonKey) =>
                setBackend((current) => ({ ...current, supabaseAnonKey }))
              }
            />
            <TextInput
              label="Access token"
              type="password"
              value={backend.supabaseAccessToken}
              onChange={(supabaseAccessToken) =>
                setBackend((current) => ({ ...current, supabaseAccessToken }))
              }
            />
          </section>

          <section className="flex flex-col gap-2">
            <SectionTitle>Workspace</SectionTitle>
            <TextInput
              label="Organization ID"
              value={workspace.organizationId}
              onChange={(organizationId) =>
                setWorkspace((current) => ({ ...current, organizationId }))
              }
            />
            <TextInput
              label="Project ID"
              value={workspace.projectId}
              onChange={(projectId) =>
                setWorkspace((current) => ({ ...current, projectId }))
              }
            />
            <TextInput
              label="Episode ID"
              value={workspace.episodeId}
              onChange={(episodeId) =>
                setWorkspace((current) => ({ ...current, episodeId }))
              }
            />
            <TextInput
              label="Stage ID"
              value={workspace.stageId}
              onChange={(stageId) =>
                setWorkspace((current) => ({ ...current, stageId }))
              }
            />
            <TextInput
              label="User ID"
              value={workspace.createdBy}
              onChange={(createdBy) =>
                setWorkspace((current) => ({ ...current, createdBy }))
              }
            />
          </section>

          <section className="flex flex-col gap-2">
            <SectionTitle>Tools</SectionTitle>
            <TextInput
              label="Renderer bin"
              value={tools.rendererBin}
              onChange={(rendererBin) =>
                setTools((current) => ({ ...current, rendererBin }))
              }
            />
            <TextInput
              label="avifenc bin"
              value={tools.avifencBin}
              onChange={(avifencBin) =>
                setTools((current) => ({ ...current, avifencBin }))
              }
            />
          </section>
        </div>
      </aside>

      <div className="grid min-h-0 min-w-0 grid-rows-[auto_1fr]">
        <header className="border-border flex items-center justify-between border-b px-5 py-3">
          <div>
            <h2 className="text-sm font-semibold">제출물 처리</h2>
            <p className="text-muted-foreground text-xs">
              원본 선택, local 변환, R2 업로드, Supabase finalize
            </p>
          </div>
          <Button variant="outline" onClick={() => window.location.reload()}>
            <ArrowClockwise />
            새로고침
          </Button>
        </header>

        <div className="grid min-h-0 min-w-0 grid-cols-[minmax(360px,0.95fr)_minmax(360px,1.05fr)] gap-0">
          <section className="border-border flex min-h-0 min-w-0 flex-col gap-4 border-r p-5">
            <PanelTitle
              title="그림 원본"
              description=".clip, .psd, .psb, .jpg, .png, .webp, .avif"
            />
            <div className="flex gap-2">
              <Button variant="outline" onClick={selectSourceFile}>
                <FolderOpen />
                파일 선택
              </Button>
              <Button
                onClick={processSelectedSource}
                disabled={!canProcess || processing}
              >
                <FileArrowUp />
                {processing ? "처리 중" : "변환 및 업로드"}
              </Button>
            </div>

            {sourceFile ? (
              <dl className="grid grid-cols-[112px_1fr] gap-x-3 gap-y-2 border border-border p-3 text-xs">
                <dt className="text-muted-foreground">파일</dt>
                <dd className="min-w-0 truncate">{sourceFile.fileName}</dd>
                <dt className="text-muted-foreground">포맷</dt>
                <dd>{sourceFile.format}</dd>
                <dt className="text-muted-foreground">크기</dt>
                <dd>{formatBytes(sourceFile.sizeBytes)}</dd>
                <dt className="text-muted-foreground">SHA-256</dt>
                <dd className="min-w-0 truncate font-mono">
                  {sourceFile.sha256}
                </dd>
              </dl>
            ) : (
              <EmptyPanel text="선택된 원본 파일이 없습니다." />
            )}

            {result ? (
              <dl className="grid grid-cols-[144px_1fr] gap-x-3 gap-y-2 border border-border bg-muted/20 p-3 text-xs">
                <dt className="text-muted-foreground">asset</dt>
                <dd className="min-w-0 truncate font-mono">{result.assetId}</dd>
                <dt className="text-muted-foreground">review asset</dt>
                <dd className="min-w-0 truncate font-mono">
                  {result.reviewAssetId}
                </dd>
                <dt className="text-muted-foreground">processing job</dt>
                <dd className="min-w-0 truncate font-mono">
                  {result.assetProcessingJobId}
                </dd>
                <dt className="text-muted-foreground">tile count</dt>
                <dd>{result.tileCount}</dd>
                <dt className="text-muted-foreground">rendered size</dt>
                <dd>
                  {result.renderedWidth} x {result.renderedHeight}
                </dd>
                <dt className="text-muted-foreground">original key</dt>
                <dd className="min-w-0 truncate font-mono">
                  {result.originalKey}
                </dd>
              </dl>
            ) : null}

            {error ? <ErrorPanel message={error} /> : null}
          </section>

          <section className="flex min-h-0 min-w-0 flex-col gap-4 p-5">
            <PanelTitle
              title="WYSIWYG 문서"
              description="외부 문서 업로드 없이 앱에서 작성과 수정만 지원"
            />
            <div className="flex gap-2">
              <Button
                variant="outline"
                size="icon"
                aria-label="Bold"
                onClick={() => formatDocument("bold")}
              >
                <TextB />
              </Button>
              <Button
                variant="outline"
                size="icon"
                aria-label="Italic"
                onClick={() => formatDocument("italic")}
              >
                <TextItalic />
              </Button>
              <Button
                variant="outline"
                onClick={loadDocument}
                disabled={loadingDocument}
              >
                <ArrowClockwise />
                {loadingDocument ? "불러오는 중" : "불러오기"}
              </Button>
              <Button onClick={saveDocument} disabled={savingDocument}>
                <FloppyDisk />
                {savingDocument ? "저장 중" : "저장"}
              </Button>
            </div>
            {/* biome-ignore lint/a11y/useSemanticElements: contentEditable is required for MVP rich text commands. */}
            <div
              ref={editorRef}
              contentEditable
              tabIndex={0}
              role="textbox"
              aria-label="WYSIWYG document editor"
              aria-multiline="true"
              className="min-h-[320px] flex-1 overflow-auto border border-border bg-background p-4 text-sm leading-7 outline-none focus:border-ring focus:ring-1 focus:ring-ring/50"
              suppressContentEditableWarning
            />
          </section>
        </div>
      </div>
    </main>
  );
}

function TextInput({
  label,
  value,
  onChange,
  type = "text",
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  type?: "text" | "password";
}) {
  return (
    <label className="flex flex-col gap-1 text-xs">
      <span className="text-muted-foreground">{label}</span>
      <input
        className="h-8 border border-input bg-background px-2 font-mono text-xs outline-none focus:border-ring focus:ring-1 focus:ring-ring/50"
        type={type}
        value={value}
        onChange={(event) => onChange(event.currentTarget.value)}
      />
    </label>
  );
}

function SectionTitle({ children }: { children: string }) {
  return (
    <h3 className="text-xs font-semibold uppercase tracking-normal text-foreground">
      {children}
    </h3>
  );
}

function PanelTitle({
  title,
  description,
}: {
  title: string;
  description: string;
}) {
  return (
    <div>
      <h3 className="text-sm font-semibold">{title}</h3>
      <p className="text-muted-foreground text-xs">{description}</p>
    </div>
  );
}

function StatusPill({ label, value }: { label: string; value: string }) {
  return (
    <div className="border border-border bg-background px-2 py-1">
      <div className="text-muted-foreground">{label}</div>
      <div className="font-mono">{value}</div>
    </div>
  );
}

function EmptyPanel({ text }: { text: string }) {
  return (
    <div className="border border-dashed border-border p-4 text-muted-foreground text-xs">
      {text}
    </div>
  );
}

function ErrorPanel({ message }: { message: string }) {
  return (
    <pre className="max-h-48 overflow-auto border border-destructive/30 bg-destructive/10 p-3 text-destructive text-xs whitespace-pre-wrap">
      {message}
    </pre>
  );
}

function emptyStringsToUndefined(tools: ToolConfig) {
  return {
    rendererBin: tools.rendererBin.trim() || undefined,
    avifencBin: tools.avifencBin.trim() || undefined,
  };
}

function formatBytes(bytes: number) {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  if (bytes < 1024 * 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}
