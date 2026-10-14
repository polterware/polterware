import { For, type JSX } from "solid-js";

/** Isometric line art for the principle row. Opaque fills so back edges hide. */
const C = Math.cos(Math.PI / 6);
const S = Math.sin(Math.PI / 6);
const iso = (x: number, y: number, z: number) => `${((x - y) * C).toFixed(1)},${((x + y) * S - z).toFixed(1)}`;

/** A box from (x, y, z) with size (w, d, h): top, left and right faces. */
function Box(props: { x: number; y: number; z: number; w: number; d: number; h: number; accent?: boolean }) {
  const { x, y, z, w, d, h } = props;
  const top = [iso(x, y, z + h), iso(x + w, y, z + h), iso(x + w, y + d, z + h), iso(x, y + d, z + h)].join(" ");
  const left = [iso(x, y + d, z + h), iso(x + w, y + d, z + h), iso(x + w, y + d, z), iso(x, y + d, z)].join(" ");
  const right = [iso(x + w, y, z + h), iso(x + w, y + d, z + h), iso(x + w, y + d, z), iso(x + w, y, z)].join(" ");
  return (
    <g class="fig-box" classList={{ "fig-accent": props.accent }}>
      <polygon points={left} class="fig-side" />
      <polygon points={right} class="fig-side" />
      <polygon points={top} class="fig-top" />
    </g>
  );
}

function Frame(props: { label: string; children: JSX.Element }) {
  return (
    <svg class="fig" viewBox="-170 -150 340 300" role="img" aria-label={props.label}>
      {props.children}
    </svg>
  );
}

/** Stacked layers: one local store under everything. */
export function FigureLayers() {
  return (
    <Frame label="Stacked layers">
      <For each={[0, 1, 2, 3, 4]}>{(i) => <Box x={-60} y={-60} z={-70 + i * 18} w={120} d={120} h={10} accent={i === 4} />}</For>
      <ellipse cx="0" cy={-64} rx="44" ry="24" class="fig-mark" />
      <ellipse cx="0" cy={-64} rx="28" ry="15" class="fig-mark" />
    </Frame>
  );
}

/** Linked blocks: facts tied to their evidence. */
export function FigureBlocks() {
  const blocks = [
    { x: -70, y: -10 }, { x: -10, y: -70 }, { x: 10, y: 10 }, { x: 50, y: -50 },
  ];
  return (
    <Frame label="Linked blocks">
      <For each={blocks}>{(b, i) => <Box x={b.x} y={b.y} z={-40} w={50} d={50} h={i() === 2 ? 70 : 50} accent={i() === 2} />}</For>
    </Frame>
  );
}

/** Fanned planes: one context handed to many agents. */
export function FigurePlanes() {
  return (
    <Frame label="Fanned planes">
      <For each={[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]}>{(i) => (
        <Box x={-80 + i * 14} y={-30} z={-70} w={4} d={90} h={40 + i * 9} accent={i === 9} />
      )}</For>
    </Frame>
  );
}
