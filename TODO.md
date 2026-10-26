# TODO

## Main

- [ ] ImGui colour change
- [ ] Virtual Geometry Streaming (Load in what we see, unload what we don't need)
- [ ] Clusters (Turning multiple triangles into a "block"
      which then can be culled instead of checking for each individual triangle.)
- [ ] Visibility Buffer (Store normals / UVs based on triangle, not per pixel)
- [ ] ZSTD for VRAM upload bandwidth reduction (So we can send more data at once)
- [ ] SSE (Decides when a certain LOD level should be visible)
- [ ] HZB culling / Occlusion culling (Why render what we can't see
      behind object anyways?)
- [ ] Compute SW Raster (Turning single pixels into actual single pixels
      so we don't waste computing power)

## Experimental

- [ ] Object relative vertices clusters
      (Subdivide the object into multiple vertices clusters,
      then get the origin of each of the clusters and get the
      vertices position relative to cluster location, and then
      get the cluster location relative to the object.)
      (Edit: These are called meshlets...)
