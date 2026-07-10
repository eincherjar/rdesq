const TAURI = typeof window !== "undefined" && window.__TAURI__
const invoke = TAURI ? window.__TAURI__.core.invoke : async () => { throw new Error("Not running in Tauri") }

window.API = {
  // ─── CONNECTIONS ─────────────────────────────
  listConnections: () => invoke("list_connections"),
  getConnection: (id) => invoke("get_connection", { id }),
  createConnection: (data) => invoke("create_connection", { data }),
  updateConnection: (id, data) => invoke("update_connection", { id, data }),
  deleteConnection: (id) => invoke("delete_connection", { id }),
  duplicateConnection: (id) => invoke("duplicate_connection", { id }),
  setFavorite: (id, favorite) => invoke("set_favorite", { id, favorite }),

  launchConnection: (id) => invoke("launch_connection", { id }),
  // ─── GROUPS ───────────────────────────────────
  listGroups: () => invoke("list_groups"),
  createGroup: (data) => invoke("create_group", { data }),
  updateGroup: (id, data) => invoke("update_group", { id, data }),
  deleteGroup: (id) => invoke("delete_group", { id }),
  updateGroupOrder: (orders) => invoke("update_group_order", { orders }),

  // ─── SETTINGS ─────────────────────────────────
  getSettings: () => invoke("get_settings"),
  saveSettings: (data) => invoke("save_settings", { data }),

  // ─── TAGS ─────────────────────────────────────
  listTags: () => invoke("list_tags"),
  renameTag: (oldName, newName) => invoke("rename_tag", { oldName, newName }),
  deleteTag: (name) => invoke("delete_tag", { name }),

  // ─── PING ─────────────────────────────────────
  pingHosts: (targets) => invoke("ping_hosts", { targets }),

  // ─── EXPORT / IMPORT ──────────────────────────
  exportData: () => invoke("export_data"),
  importData: (data) => invoke("import_data", { data }),
  saveExportFile: (data) => invoke("save_export_file", { data }),
}
