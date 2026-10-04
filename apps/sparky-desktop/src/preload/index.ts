import { contextBridge } from "electron";

contextBridge.exposeInMainWorld("sparky", {
  app: {
    name: "Sparky",
  },
});
