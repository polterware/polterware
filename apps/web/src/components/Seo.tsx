import { Title, Meta, Link } from "@solidjs/meta";
export function Seo(props: { title: string; description: string; path: string; noindex?: boolean }) {
  const title = () => props.title === "polterware" ? props.title : `${props.title} · polterware`;
  const url = () => `https://polterware.com${props.path}`;
  return <>
    <Title>{title()}</Title>
    <Meta name="description" content={props.description} />
    <Link rel="canonical" href={url()} />
    <Meta property="og:type" content="website" />
    <Meta property="og:title" content={title()} />
    <Meta property="og:description" content={props.description} />
    <Meta property="og:url" content={url()} />
    <Meta property="og:image" content="https://polterware.com/polter-ghost.png" />
    <Meta name="twitter:card" content="summary" />
    <Meta name="twitter:title" content={title()} />
    <Meta name="twitter:description" content={props.description} />
    <Meta name="twitter:image" content="https://polterware.com/polter-ghost.png" />
    {props.noindex && <Meta name="robots" content="noindex" />}
  </>;
}
