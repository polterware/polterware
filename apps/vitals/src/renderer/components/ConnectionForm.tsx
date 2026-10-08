import { Button, Card, Checkbox, Delete, Field, Input, Select } from "@polterware/frontend-design-system";
import { createSignal, For, onMount, Show } from "solid-js";
import type { AuthMode, ConnectPayload, SavedServerPublic } from "../api";

export function ConnectionForm(props: {
  saved: SavedServerPublic[];
  onConnect: (payload: ConnectPayload) => void;
  onDeleteSaved: (id: string) => void;
  onRefreshSaved: () => void;
}) {
  const [label, setLabel] = createSignal("");
  const [host, setHost] = createSignal("");
  const [port, setPort] = createSignal(22);
  const [username, setUsername] = createSignal("root");
  const [authMode, setAuthMode] = createSignal<AuthMode>("password");
  const [password, setPassword] = createSignal("");
  const [privateKeyPath, setPrivateKeyPath] = createSignal("");
  const [passphrase, setPassphrase] = createSignal("");
  const [save, setSave] = createSignal(true);

  onMount(() => props.onRefreshSaved());

  const submit = (event: SubmitEvent) => {
    event.preventDefault();
    const mode = authMode();
    props.onConnect({
      label: label() || host(),
      host: host(),
      port: port(),
      username: username(),
      authMode: mode,
      password: mode === "password" ? password() : undefined,
      privateKeyPath: mode === "privateKey" ? privateKeyPath() : undefined,
      privateKeyPassphrase: mode === "privateKey" ? passphrase() : undefined,
      save: save(),
    });
  };

  const loadSaved = (server: SavedServerPublic) => {
    setLabel(server.label);
    setHost(server.host);
    setPort(server.port);
    setUsername(server.username);
    setAuthMode(server.authMode);
    setPrivateKeyPath(server.privateKeyPath ?? "");
    props.onConnect({
      serverId: server.id,
      label: server.label,
      host: server.host,
      port: server.port,
      username: server.username,
      authMode: server.authMode,
      privateKeyPath: server.privateKeyPath,
    });
  };

  return (
    <form class="vitals-form" onSubmit={submit}>
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
      <Checkbox
        checked={save()}
        onCheckedChange={(details) => setSave(details.checked === true)}
        label="Save server (secrets encrypted with OS keychain)"
      />
      <Button type="submit" variant="brand" style={{ width: "100%" }}>
        Connect
      </Button>

      <div class="vitals-saved">
        <h3 class="vitals-card-title">Saved servers</h3>
        <Show when={props.saved.length === 0}>
          <p class="vitals-sub">No saved servers yet.</p>
        </Show>
        <For each={props.saved}>
          {(server) => (
            <Card>
              <div class="vitals-saved-row">
                <Button variant="ghost" onClick={() => loadSaved(server)}>
                  {server.label}
                </Button>
                <Button variant="danger" size="sm" onClick={() => props.onDeleteSaved(server.id)} aria-label={`Remove ${server.label}`}>
                  <Delete size={14} />
                  Remove
                </Button>
              </div>
            </Card>
          )}
        </For>
      </div>
    </form>
  );
}
