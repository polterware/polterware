import {
  A11yDialog,
  ControlButton,
  createFluidHover,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Delete,
  Field,
  FluidHoverHighlight,
  Input,
  MoreHorizontal,
  Pencil,
  Plus,
  Select,
  Server,
  SwitchRow,
} from "@polterware/frontend-design-system";
import { SidebarSection } from "@polterware/frontend-shell";
import { createEffect, createSignal, Index, onCleanup, onMount, Show } from "solid-js";
import type { AuthMode, ConnectPayload, SavedServerPublic } from "../api";

export function savedConnectPayload(server: SavedServerPublic): ConnectPayload {
  return {
    serverId: server.id,
    label: server.label,
    host: server.host,
    port: server.port,
    username: server.username,
    authMode: server.authMode,
    privateKeyPath: server.privateKeyPath,
  };
}

function SavedServerRow(props: {
  server: SavedServerPublic;
  index: number;
  hover: { registerItem: (index: number, element: HTMLElement) => () => void };
  onOpen: () => void;
  onEdit: () => void;
  onRemove: () => void;
}) {
  let row: HTMLDivElement | undefined;
  onMount(() => {
    const element = row;
    if (!element) return;
    createEffect(() => onCleanup(props.hover.registerItem(props.index, element)));
  });
  return (
    <div ref={row} class="pw-sidebar-item pw-sidebar-item--nav vitals-saved-row">
      <button type="button" class="vitals-saved-name" onClick={() => props.onOpen()}>
        <Server size={14} aria-hidden />
        <span class="vitals-saved-label">{props.server.label}</span>
      </button>
      <DropdownMenu positioning={{ placement: "bottom-end" }}>
        <DropdownMenuTrigger class="vitals-saved-more" aria-label={`Actions for ${props.server.label}`}>
          <MoreHorizontal size={14} />
        </DropdownMenuTrigger>
        <DropdownMenuContent class="min-w-40">
          <DropdownMenuItem value="edit" onSelect={() => props.onEdit()}>
            <Pencil size={14} aria-hidden />
            Edit
          </DropdownMenuItem>
          <DropdownMenuItem value="remove" danger onSelect={() => props.onRemove()}>
            <Delete size={14} aria-hidden />
            Remove
          </DropdownMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
}

export function ConnectionForm(props: {
  saved: SavedServerPublic[];
  onConnect: (payload: ConnectPayload) => void;
  onDeleteSaved: (id: string) => void;
  onRefreshSaved: () => void;
  bindOpenNew?: (open: () => void) => void;
}) {
  const [open, setOpen] = createSignal(false);
  const [editingId, setEditingId] = createSignal<string | null>(null);
  const [label, setLabel] = createSignal("");
  const [host, setHost] = createSignal("");
  const [port, setPort] = createSignal(22);
  const [username, setUsername] = createSignal("root");
  const [authMode, setAuthMode] = createSignal<AuthMode>("password");
  const [password, setPassword] = createSignal("");
  const [privateKeyPath, setPrivateKeyPath] = createSignal("");
  const [passphrase, setPassphrase] = createSignal("");
  const [save, setSave] = createSignal(true);
  let savedList: HTMLDivElement | undefined;
  const savedHover = createFluidHover(() => savedList, { gapClick: { maxDistance: 8 } });

  onMount(() => props.onRefreshSaved());
  props.bindOpenNew?.(() => openNew());

  const canSubmit = () => host().trim().length > 0 && username().trim().length > 0;

  const reset = () => {
    setEditingId(null);
    setLabel("");
    setHost("");
    setPort(22);
    setUsername("root");
    setAuthMode("password");
    setPassword("");
    setPrivateKeyPath("");
    setPassphrase("");
    setSave(true);
  };

  const openNew = () => {
    reset();
    setOpen(true);
  };

  const openEdit = (server: SavedServerPublic) => {
    setEditingId(server.id);
    setLabel(server.label);
    setHost(server.host);
    setPort(server.port);
    setUsername(server.username);
    setAuthMode(server.authMode);
    setPrivateKeyPath(server.privateKeyPath ?? "");
    setPassword("");
    setPassphrase("");
    setSave(true);
    setOpen(true);
  };

  const close = () => setOpen(false);

  const submit = () => {
    if (!canSubmit()) return;
    const mode = authMode();
    const editing = editingId();
    props.onConnect({
      serverId: editing ?? undefined,
      label: label() || host(),
      host: host(),
      port: port(),
      username: username(),
      authMode: mode,
      password: mode === "password" ? password() : undefined,
      privateKeyPath: mode === "privateKey" ? privateKeyPath() : undefined,
      privateKeyPassphrase: mode === "privateKey" ? passphrase() : undefined,
      save: editing ? true : save(),
    });
    props.onRefreshSaved();
    close();
  };

  const loadSaved = (server: SavedServerPublic) => {
    props.onConnect(savedConnectPayload(server));
  };

  return (
    <div class="vitals-saved">
      <div class="vitals-saved-scroll">
      <SidebarSection label="Saved servers">
        <Show when={props.saved.length === 0}>
          <p class="vitals-sub">No saved servers yet.</p>
        </Show>
        <div class="vitals-saved-list" ref={savedList} {...savedHover.handlers}>
          <FluidHoverHighlight hover={savedHover} class="pw-sidebar-hover" />
          <Index each={props.saved}>
            {(server, index) => (
              <SavedServerRow
                server={server()}
                index={index}
                hover={savedHover}
                onOpen={() => loadSaved(server())}
                onEdit={() => openEdit(server())}
                onRemove={() => props.onDeleteSaved(server().id)}
              />
            )}
          </Index>
        </div>
      </SidebarSection>
      </div>
      <ControlButton class="vitals-connect" tone="brand" onClick={openNew}>
        <Plus size={14} />
        Connect
      </ControlButton>
      <A11yDialog
        open={open()}
        size="sm"
        dividers={false}
        title={editingId() ? "Edit server" : "Connect"}
        onClose={close}
        footer={
          <>
            <ControlButton class="vitals-dialog-cancel" tone="secondary" onClick={close}>
              Cancel
            </ControlButton>
            <ControlButton tone="brand" disabled={!canSubmit()} onClick={submit}>
              Connect
            </ControlButton>
          </>
        }
      >
        <div class="vitals-form">
          <Field label="Label" for="vitals-label">
            <Input id="vitals-label" value={label()} placeholder="prod-1" onInput={(event) => setLabel(event.currentTarget.value)} />
          </Field>
          <Field label="Host" for="vitals-host">
            <Input id="vitals-host" value={host()} required placeholder="192.168.1.10" onInput={(event) => setHost(event.currentTarget.value)} />
          </Field>
          <Field label="Port" for="vitals-port">
            <Input
              id="vitals-port"
              type="number"
              min={1}
              max={65535}
              value={port()}
              onInput={(event) => setPort(Number(event.currentTarget.value))}
            />
          </Field>
          <Field label="User" for="vitals-user">
            <Input id="vitals-user" value={username()} required onInput={(event) => setUsername(event.currentTarget.value)} />
          </Field>
          <Field label="Authentication">
            <Select
              aria-label="Authentication"
              value={authMode()}
              onChange={(value) => setAuthMode(value as AuthMode)}
              options={[
                { value: "password", label: "Password" },
                { value: "privateKey", label: "Private key" },
              ]}
            />
          </Field>
          <Show
            when={authMode() === "password"}
            fallback={
              <>
                <Field label="Private key path" for="vitals-key">
                  <Input
                    id="vitals-key"
                    value={privateKeyPath()}
                    placeholder="/Users/you/.ssh/id_ed25519"
                    onInput={(event) => setPrivateKeyPath(event.currentTarget.value)}
                  />
                </Field>
                <Field label="Passphrase (optional)" for="vitals-passphrase">
                  <Input
                    id="vitals-passphrase"
                    type="password"
                    value={passphrase()}
                    onInput={(event) => setPassphrase(event.currentTarget.value)}
                  />
                </Field>
              </>
            }
          >
            <Field label="Password" for="vitals-password">
              <Input id="vitals-password" type="password" value={password()} onInput={(event) => setPassword(event.currentTarget.value)} />
            </Field>
          </Show>
          <Show when={!editingId()}>
            <SwitchRow
              checked={save()}
              onCheckedChange={setSave}
              label="Save server (secrets encrypted with OS keychain)"
            />
          </Show>
        </div>
      </A11yDialog>
    </div>
  );
}
