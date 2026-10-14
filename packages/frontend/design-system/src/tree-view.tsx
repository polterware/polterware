import { TreeView as ArkTreeView, createTreeCollection, type TreeViewRootProps } from "@ark-ui/solid/tree-view";
import { For, Show, createMemo, splitProps, type JSX } from "solid-js";
import { ChevronRight } from "./hugeicons";
import "./tree-view.css";

export type TreeViewNode = { id: string; label: string; children?: TreeViewNode[]; icon?: JSX.Element; trailing?: JSX.Element };
export type TreeViewProps = Omit<TreeViewRootProps<TreeViewNode>, "children" | "collection"> & {
  nodes: TreeViewNode[];
  label: string;
};

export function TreeView(props: TreeViewProps) {
  const [local, rest] = splitProps(props, ["nodes", "label", "class"]);
  const collection = createMemo(() =>
    createTreeCollection<TreeViewNode>({
      rootNode: { id: "__root__", label: "", children: local.nodes },
      nodeToValue: (node) => node.id,
      nodeToString: (node) => node.label,
    }),
  );
  return (
    <ArkTreeView.Root {...rest} collection={collection()} class={`pw-tree ${local.class ?? ""}`}>
      <ArkTreeView.Label>{local.label}</ArkTreeView.Label>
      <ArkTreeView.Tree>
        <For each={local.nodes}>{(node, index) => <TreeNode node={node} indexPath={[index()]} />}</For>
      </ArkTreeView.Tree>
    </ArkTreeView.Root>
  );
}

function TreeNode(props: { node: TreeViewNode; indexPath: number[] }) {
  return (
    <ArkTreeView.NodeProvider node={props.node} indexPath={props.indexPath}>
      <Show
        when={props.node.children?.length}
        fallback={
          <ArkTreeView.Item class="pw-tree__row">
            {props.node.icon}
            <ArkTreeView.ItemText class="pw-tree__text">{props.node.label}</ArkTreeView.ItemText>
            {props.node.trailing ? <span class="pw-tree__trailing">{props.node.trailing}</span> : null}
          </ArkTreeView.Item>
        }
      >
        <ArkTreeView.Branch>
          <ArkTreeView.BranchControl class="pw-tree__row">
            {props.node.icon}
            <ArkTreeView.BranchText class="pw-tree__text">{props.node.label}</ArkTreeView.BranchText>
            <ArkTreeView.BranchIndicator class="pw-tree__indicator">
              <ChevronRight size={14} />
            </ArkTreeView.BranchIndicator>
            {props.node.trailing ? <span class="pw-tree__trailing pw-tree__trailing--after-indicator">{props.node.trailing}</span> : null}
          </ArkTreeView.BranchControl>
          <ArkTreeView.BranchContent class="pw-tree__children">
            <ArkTreeView.BranchIndentGuide class="pw-tree__guide" />
            <For each={props.node.children}>
              {(node, index) => <TreeNode node={node} indexPath={[...props.indexPath, index()]} />}
            </For>
          </ArkTreeView.BranchContent>
        </ArkTreeView.Branch>
      </Show>
    </ArkTreeView.NodeProvider>
  );
}
