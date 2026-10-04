declare module "supercluster" {
  type PointFeature<P> = {
    type: "Feature";
    properties: P;
    geometry: { type: "Point"; coordinates: [number, number] };
  };

  export default class Supercluster<P extends object = { [key: string]: unknown }> {
    constructor(options?: { radius?: number; maxZoom?: number; minPoints?: number });
    load(points: Array<PointFeature<P>>): this;
    getClusters(bbox: [number, number, number, number], zoom: number): Array<PointFeature<P | {
      cluster: true;
      cluster_id: number;
      point_count: number;
      point_count_abbreviated: string | number;
    }>>;
    getClusterExpansionZoom(clusterId: number): number;
  }
}
