

let connections = []
let groups = []
let currentFilter = ""
let sortField = "name"
let sortAsc = true
let collapsedSections = new Set()
let settings = { lang: "pl", theme: "dark", uiScale: 1, startWithSystem: false, startMinimized: false, closeToTray: false }
let confirmCallback = null
let pingCache = new Map()
let pinging = false
let appWin = null

function esc(str) {
  if (typeof str !== "string") return ""
  const map = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#x27;" }
  return str.replace(/[&<>"']/g, (c) => map[c] || c)
}

function debounce(fn, ms) {
  let timer
  return (...args) => {
    clearTimeout(timer)
    timer = setTimeout(() => fn(...args), ms)
  }
}

const FOLDER_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M5 4h4l3 3h7a2 2 0 0 1 2 2v8a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2"/></svg>`
const FOLDER_OPEN_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M5 19l2.757 -7.351a1 1 0 0 1 .936 -.649h12.307a1 1 0 0 1 .986 1.164l-.996 5.211a2 2 0 0 1 -1.964 1.625h-14.026a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2h4l3 3h7a2 2 0 0 1 2 2v2"/></svg>`
const FOLDER_OFF_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M13.5 19h-8.5a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2h4l3 3h7a2 2 0 0 1 2 2v4"/><path d="M22 22l-5 -5"/><path d="M17 22l5 -5"/></svg>`
const CHEVRON_DOWN = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M6 9l6 6l6 -6"/></svg>`
const CHEVRON_RIGHT = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M9 6l6 6l-6 6"/></svg>`
const BOOKMARK_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M18 7v14l-6 -4l-6 4v-14a4 4 0 0 1 4 -4h4a4 4 0 0 1 4 4"/></svg>`
const BOOKMARK_FILLED_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="currentColor" class="icon-tabler-filled"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M14 2a5 5 0 0 1 5 5v14a1 1 0 0 1 -1.555 .832l-5.445 -3.63l-5.444 3.63a1 1 0 0 1 -1.55 -.72l-.006 -.112v-14a5 5 0 0 1 5 -5h4z"/></svg>`
const PLAY_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M7 4v16l13 -8l-13 -8"/></svg>`
const PENCIL_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M4 20h4l10.5 -10.5a2.828 2.828 0 1 0 -4 -4l-10.5 10.5v4"/><path d="M13.5 6.5l4 4"/></svg>`
const COPY_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M7 9.667a2.667 2.667 0 0 1 2.667 -2.667h8.666a2.667 2.667 0 0 1 2.667 2.667v8.666a2.667 2.667 0 0 1 -2.667 2.667h-8.666a2.667 2.667 0 0 1 -2.667 -2.667l0 -8.666"/><path d="M4.012 16.737a2.005 2.005 0 0 1 -1.012 -1.737v-10c0 -1.1 .9 -2 2 -2h10c.75 0 1.158 .385 1.5 1"/></svg>`
const TRASH_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M4 7l16 0"/><path d="M10 11l0 6"/><path d="M14 11l0 6"/><path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12"/><path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3"/></svg>`
const SSH_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M8 9l3 3l-3 3"/><path d="M13 15l3 0"/><path d="M3 6a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2l0 -12"/></svg>`
const RDP_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M3 5a1 1 0 0 1 1 -1h16a1 1 0 0 1 1 1v10a1 1 0 0 1 -1 1h-16a1 1 0 0 1 -1 -1v-10"/><path d="M7 20h10"/><path d="M9 16v4"/><path d="M15 16v4"/></svg>`
const PLUS_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 5l0 14"/><path d="M5 12l14 0"/></svg>`
const GROUP_ADD_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 19h-7a2 2 0 0 1 -2 -2v-11a2 2 0 0 1 2 -2h4l3 3h7a2 2 0 0 1 2 2v3.5"/><path d="M16 19h6"/><path d="M19 16v6"/></svg>`
const LIGHTNING_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M13 3l0 7l6 0l-8 11l0 -7l-6 0l8 -11"/></svg>`
const SORT_UP_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 5l0 14"/><path d="M18 11l-6 -6"/><path d="M6 11l6 -6"/></svg>`
const SORT_DOWN_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M12 5l0 14"/><path d="M18 13l-6 6"/><path d="M6 13l6 6"/></svg>`
const EYE_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M10 12a2 2 0 1 0 4 0a2 2 0 0 0 -4 0"/><path d="M21 12c-2.4 4 -5.4 6 -9 6c-3.6 0 -6.6 -2 -9 -6c2.4 -4 5.4 -6 9 -6c3.6 0 6.6 2 9 6"/></svg>`
const EYE_OFF_ICON = `<svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path stroke="none" d="M0 0h24v24H0z" fill="none"/><path d="M10.585 10.587a2 2 0 0 0 2.829 2.828"/><path d="M16.681 16.673a8.717 8.717 0 0 1 -4.681 1.327c-3.6 0 -6.6 -2 -9 -6c1.272 -2.12 2.712 -3.678 4.32 -4.674m2.86 -1.146a9.055 9.055 0 0 1 1.82 -.18c3.6 0 6.6 2 9 6c-.666 1.11 -1.379 2.067 -2.138 2.87"/><path d="M3 3l18 18"/></svg>`

function sanitizeId(id) {
  return id.replace(/['"\\]/g, "")
}

async function loadData() {
  try {
    const [conns, grps, s] = await Promise.all([
      API.listConnections(),
      API.listGroups(),
      API.getSettings(),
    ])
    connections = conns
    groups = grps
    settings = s
    applySettings()
    render()
    pingHosts(connections)

    if (appWin && !settings.startMinimized) {
      appWin.show().catch(() => {})
      appWin.setFocus().catch(() => {})
    }
  } catch (err) {
    showToast(err.message, "error")
  }
}

function applySettings() {
  setLang(settings.lang)
  document.documentElement.setAttribute("data-theme", settings.theme)
  const app = document.querySelector(".app")
  app.style.zoom = settings.uiScale
  app.style.height = `calc(100vh / ${settings.uiScale})`
  document.getElementById("pageTitle").textContent = t("appTitle")
  updateI18n()
  updateSortBtn()
}

function updateI18n() {
  document.querySelectorAll("[data-i18n]").forEach(el => {
    el.textContent = t(el.dataset.i18n)
  })
  document.querySelectorAll("[data-i18n-placeholder]").forEach(el => {
    el.placeholder = t(el.dataset.i18nPlaceholder)
  })
  document.querySelectorAll("[data-i18n-value]").forEach(el => {
    el.value = t(el.dataset.i18nValue)
  })
  document.querySelectorAll("[data-i18n-title]").forEach(el => {
    const text = t(el.dataset.i18nTitle)
    el.dataset.tooltip = text
    el.setAttribute("aria-label", text)
  })
}

// ─── RENDER ─────────────────────────────────────

function render() {
  const main = document.getElementById("mainContent")
  let filtered = connections

  if (currentFilter) {
    filtered = connections.filter(c =>
      c.name.toLowerCase().includes(currentFilter) ||
      c.host.toLowerCase().includes(currentFilter) ||
      c.tags.some(t => t.toLowerCase().includes(currentFilter))
    )
  }

  const sorted = [...filtered].sort((a, b) => {
    let cmp
    if (sortField === "host") cmp = a.host.localeCompare(b.host)
    else if (sortField === "protocol") cmp = a.protocol.localeCompare(b.protocol)
    else cmp = a.name.localeCompare(b.name)
    return sortAsc ? cmp : -cmp
  })

  const favorites = sorted.filter(c => c.favorite)
  const ungrouped = sorted.filter(c => !c.groupId && !c.favorite)

  const grouped = new Map()
  for (const g of groups) {
    const gc = sorted.filter(c => c.groupId === g.id && !c.favorite)
    grouped.set(g.id, { group: g, connections: gc })
  }

  const sections = []
  if (favorites.length > 0) sections.push({ id: "__fav", title: t("favorites"), icon: "bookmark", color: "#F59E0B", conns: favorites, collapsible: true })
  for (const [id, d] of grouped) sections.push({ id, title: d.group.name, color: d.group.color || "#9DD99A", conns: d.connections, collapsible: true, group: d.group })
  if (ungrouped.length > 0) sections.push({ id: "__nogroup", title: t("noGroup"), icon: "folder-off", color: "#94A3B8", conns: ungrouped, collapsible: true })

  if (sections.length === 0) {
    main.innerHTML = `<div class="empty-state">
      <h3>${t("noConnections")}</h3>
      <p>${t("noConnectionsDesc")}</p>
      <button class="btn btn-primary" data-action="newConnection">${t("newConnectionShort")}</button>
    </div>`
    main.querySelector("[data-action='newConnection']")?.addEventListener("click", () => showConnectionModal())
    return
  }

  main.innerHTML = sections.map(s => renderSection(s)).join("")
  main.querySelectorAll("[data-action]").forEach(el => {
    const action = el.dataset.action
    if (action === "editGroup") el.addEventListener("click", () => showGroupModal(el.dataset.id))
    else if (action === "deleteGroup") el.addEventListener("click", () => deleteGroup(el.dataset.id))
    else if (action === "toggleCollapse") el.addEventListener("click", () => toggleCollapse(el.dataset.section))
    else if (action === "favorite") el.addEventListener("click", () => toggleFavorite(el.dataset.id))
    else if (action === "connect") el.addEventListener("click", () => launchConnection(el.dataset.id))
    else if (action === "edit") el.addEventListener("click", () => showConnectionModal(el.dataset.id))
    else if (action === "delete") el.addEventListener("click", () => deleteConnection(el.dataset.id))
    else if (action === "duplicate") el.addEventListener("click", () => duplicateConnection(el.dataset.id))
  })

  main.querySelectorAll(".connection-row").forEach(row => {
    row.addEventListener("dblclick", () => launchConnection(row.dataset.id))
  })

  setupDragDrop()
}

function renderSection(s) {
  const isCollapsed = collapsedSections.has(s.id)
  const count = s.conns.length

  let editBtns = ""
  if (s.group) {
    editBtns = `<span class="section-actions">
      <button class="section-action-btn" data-action="editGroup" data-id="${sanitizeId(s.group.id)}" data-tooltip="${t("edit")}" aria-label="${t("edit")}">${PENCIL_ICON}</button>
      <button class="section-action-btn danger" data-action="deleteGroup" data-id="${sanitizeId(s.group.id)}" data-tooltip="${t("delete")}" aria-label="${t("delete")}">${TRASH_ICON}</button>
    </span>`
  }

  const rows = s.conns.map(c => renderRow(c)).join("")

  return `<div class="tree-section">
    <div class="tree-section-header ${isCollapsed ? "collapsed" : ""}"
         data-action="${s.collapsible ? "toggleCollapse" : ""}"
         data-section="${sanitizeId(s.id)}">
      ${s.collapsible ? `<span class="collapse-icon">${isCollapsed ? CHEVRON_RIGHT : CHEVRON_DOWN}</span>` : '<span style="width:14px"></span>'}
      ${s.icon === "bookmark" ? `<span class="section-icon" style="color:${esc(s.color || "var(--fav-color)")}">${BOOKMARK_FILLED_ICON}</span>` : s.icon === "folder-off" ? `<span class="section-icon" style="color:${esc(s.color || "#94A3B8")}">${FOLDER_OFF_ICON}</span>` : s.color ? `<span class="group-folder" style="color:${esc(s.color)}">${isCollapsed ? FOLDER_ICON : FOLDER_OPEN_ICON}</span>` : ""}
      <span class="section-title">${esc(s.title)}</span>
      <span class="section-count">${count}</span>
      <span class="section-spacer"></span>
      ${editBtns}
    </div>
    <div class="tree-section-body ${isCollapsed ? "collapsed" : ""}">
      ${rows}
    </div>
  </div>`
}

function renderRow(conn) {
  const id = sanitizeId(conn.id)
  const icon = conn.protocol === "ssh" ? SSH_ICON : RDP_ICON
  const pingStatus = getPingStatus(conn)
  const pingCls = pingStatus === true ? " online" : pingStatus === false ? " offline" : ""
  return `<div class="connection-row" data-id="${id}" draggable="true">
    <span class="conn-icon${pingCls}">${icon}</span>
    <span class="conn-name">${esc(conn.name)}</span>
    <span class="conn-host">${esc(conn.username)}@${esc(conn.host)}:${conn.port}</span>
    <span class="conn-protocol">${esc(conn.protocol)}</span>
    <span class="row-actions">
      <button class="row-btn fav ${conn.favorite ? "active" : ""}" data-action="favorite" data-id="${id}" data-tooltip="${t("favorite")}" aria-label="${t("favorite")}">${conn.favorite ? BOOKMARK_FILLED_ICON : BOOKMARK_ICON}</button>
      <button class="row-btn play" data-action="connect" data-id="${id}" data-tooltip="${t("connect")}" aria-label="${t("connect")}">${PLAY_ICON}</button>
      <button class="row-btn" data-action="edit" data-id="${id}" data-tooltip="${t("edit")}" aria-label="${t("edit")}">${PENCIL_ICON}</button>
      <button class="row-btn" data-action="duplicate" data-id="${id}" data-tooltip="${t("duplicate")}" aria-label="${t("duplicate")}">${COPY_ICON}</button>
      <button class="row-btn danger" data-action="delete" data-id="${id}" data-tooltip="${t("delete")}" aria-label="${t("delete")}">${TRASH_ICON}</button>
    </span>
  </div>`
}

// ─── DRAG & DROP ─────────────────────────────────

function setupDragDrop() {
  document.querySelectorAll(".connection-row[draggable]").forEach(el => {
    el.addEventListener("dragstart", onDragStart)
    el.addEventListener("dragend", onDragEnd)
  })
  document.querySelectorAll(".tree-section-body").forEach(el => {
    el.addEventListener("dragover", onDragOver)
    el.addEventListener("dragleave", onDragLeave)
    el.addEventListener("drop", onDrop)
  })
}

let draggedId = null

function onDragStart(e) {
  draggedId = e.currentTarget.dataset.id
  e.currentTarget.classList.add("dragging")
  e.dataTransfer.effectAllowed = "move"
}

function onDragEnd(e) {
  e.currentTarget.classList.remove("dragging")
  document.querySelectorAll(".drag-over").forEach(el => el.classList.remove("drag-over"))
  draggedId = null
}

function onDragOver(e) {
  e.preventDefault()
  e.dataTransfer.dropEffect = "move"
  const section = e.currentTarget.closest(".tree-section")
  section?.querySelectorAll(".connection-row").forEach(r => r.classList.remove("drag-over"))
  const target = e.currentTarget.closest(".tree-section-body")
  target?.classList.add("drag-over")
}

function onDragLeave(e) {
  e.currentTarget.classList.remove("drag-over")
}

function onDrop(e) {
  e.preventDefault()
  e.currentTarget.classList.remove("drag-over")
  if (!draggedId) return

  const sectionHeader = e.currentTarget.closest(".tree-section")?.querySelector(".tree-section-header")
  if (!sectionHeader) return

  let targetGroupId = null
  const editBtn = sectionHeader.querySelector("[data-action='editGroup']")
  if (editBtn) targetGroupId = editBtn.dataset.id

  const conn = connections.find(c => c.id === draggedId)
  if (!conn) return

  const newGroupId = targetGroupId || undefined
  if (conn.groupId !== newGroupId) {
    API.updateConnection(draggedId, { groupId: newGroupId }).then(loadData).catch(err => showToast(err.message, "error"))
  }
  draggedId = null
}

// ─── ACTIONS ─────────────────────────────────────

function toggleCollapse(sectionId) {
  if (collapsedSections.has(sectionId)) collapsedSections.delete(sectionId)
  else collapsedSections.add(sectionId)
  render()
}

function toggleFavorite(id) {
  const conn = connections.find(c => c.id === id)
  if (!conn) return
  const next = !conn.favorite
  API.setFavorite(id, next)
    .then(() => { showToast(next ? "Dodano do ulubionych" : "Usunięto z ulubionych", "success"); loadData() })
    .catch(err => showToast(err.message, "error"))
}

const filterConnections = debounce(() => {
  currentFilter = document.getElementById("searchInput").value.toLowerCase()
  render()
}, 200)

function toggleSortDir() {
  sortAsc = !sortAsc
  updateSortBtn()
  render()
}

function onSortFieldChange(value) {
  sortField = value
  closeSortDropdown()
  updateSortBtn()
  render()
}

function updateSortBtn() {
  document.getElementById("sortFieldLabel").textContent = t("field" + sortField.charAt(0).toUpperCase() + sortField.slice(1))
  document.querySelectorAll(".sort-dropdown-option").forEach(el => {
    el.classList.toggle("active", el.dataset.value === sortField)
  })
  document.getElementById("sortDirBtn").innerHTML = sortAsc ? SORT_UP_ICON : SORT_DOWN_ICON
}

function closeSortDropdown() {
  document.getElementById("sortDropdownMenu").classList.remove("open")
}

document.addEventListener("click", (e) => {
  const dd = document.getElementById("sortDropdown")
  if (dd && !dd.contains(e.target)) closeSortDropdown()
})

async function duplicateConnection(id) {
  try {
    await API.duplicateConnection(id)
    showToast(t("connectionCreated"), "success")
    await loadData()
  } catch (err) { showToast(err.message, "error") }
}

// ─── PING ─────────────────────────────────────────

function pingKey(conn) {
  return `${conn.host}:${conn.port}`
}

function getPingStatus(conn) {
  const cached = pingCache.get(pingKey(conn))
  if (cached && Date.now() - cached.t < 30000) return cached.online
  return null
}

function setPingStatus(conn, online) {
  pingCache.set(pingKey(conn), { online, t: Date.now() })
}

async function pingHosts(hostList) {
  const need = hostList.filter(c => getPingStatus(c) === null)
  if (need.length === 0) return
  pinging = true
  document.getElementById("pingAllBtn")?.classList.add("pinging")
  try {
    const results = await API.pingHosts(need.map(c => ({ host: c.host, port: c.port, protocol: c.protocol })))
    for (const r of results) {
      for (const c of hostList) {
        if (c.host === r.host && c.port === r.port) {
          setPingStatus(c, r.reachable)
        }
      }
    }
    updateIcons()
  } catch { /* silent */ }
  pinging = false
  document.getElementById("pingAllBtn")?.classList.remove("pinging")
}

async function pingAllHosts() {
  pingCache.clear()
  await pingHosts(connections)
}

function updateIcons() {
  document.querySelectorAll(".connection-row").forEach(row => {
    const id = row.dataset.id
    const conn = connections.find(c => c.id === id)
    if (!conn) return
    const icon = row.querySelector(".conn-icon")
    const status = getPingStatus(conn)
    icon.classList.toggle("online", status === true)
    icon.classList.toggle("offline", status === false)
  })
}

// ─── MODALS ──────────────────────────────────────

function openModal(id) { document.getElementById(id).style.display = "flex" }
function closeModal(id) { document.getElementById(id).style.display = "none" }

document.querySelectorAll(".modal-overlay").forEach(overlay => {
  overlay.addEventListener("mousedown", e => {
    overlay.dataset.mdTarget = e.target === overlay ? "overlay" : "content"
  })
  overlay.addEventListener("mouseup", e => {
    if (e.target === overlay && overlay.dataset.mdTarget === "overlay") {
      overlay.style.display = "none"
    }
    overlay.dataset.mdTarget = ""
  })
})

document.querySelectorAll(".modal-close").forEach(btn => {
  btn.addEventListener("click", () => {
    btn.closest(".modal-overlay").style.display = "none"
  })
})

function showConfirm(message, callback) {
  document.getElementById("confirmMessage").textContent = message
  confirmCallback = callback
  openModal("confirmModal")
}

document.getElementById("confirmYes").addEventListener("click", () => {
  closeModal("confirmModal")
  if (confirmCallback) { confirmCallback(); confirmCallback = null }
})

function showConnectionModal(id) {
  document.getElementById("connectionForm").reset()
  document.getElementById("connectionId").value = ""
  document.getElementById("connPort").value = "22"
  document.getElementById("connectionModalTitle").textContent = id ? t("editConnection") : t("newConnection")

  const groupSelect = document.getElementById("connGroup")
  groupSelect.innerHTML = `<option value="">${t("none")}</option>`
  groups.forEach(g => { groupSelect.innerHTML += `<option value="${g.id}">${esc(g.name)}</option>` })

  const tagList = document.getElementById("tagSuggestions")
  const allTags = [...new Set(connections.flatMap(c => c.tags || []))].sort()
  tagList.innerHTML = allTags.map(t => `<option value="${esc(t)}">`).join("")

  if (id) {
    const conn = connections.find(c => c.id === id)
    if (!conn) return
    document.getElementById("connectionId").value = conn.id
    document.getElementById("connName").value = conn.name
    document.getElementById("connProtocol").value = conn.protocol
    document.getElementById("connPort").value = conn.port
    document.getElementById("connHost").value = conn.host
    document.getElementById("connUsername").value = conn.username
    document.getElementById("connAuthType").value = conn.authType
    if (conn.authType === "password") document.getElementById("connPassword").value = conn.password || ""
    else document.getElementById("connKeyPath").value = conn.privateKeyPath || ""
    document.getElementById("connGroup").value = conn.groupId || ""
    document.getElementById("connTags").value = conn.tags.join(", ")
    toggleAuthFields()
    togglePort()
  }
  openModal("connectionModal")
}

async function saveConnection(e) {
  e.preventDefault()
  const id = document.getElementById("connectionId").value
  const data = {
    name: document.getElementById("connName").value,
    protocol: document.getElementById("connProtocol").value,
    port: parseInt(document.getElementById("connPort").value),
    host: document.getElementById("connHost").value,
    username: document.getElementById("connUsername").value,
    authType: document.getElementById("connAuthType").value,
    groupId: document.getElementById("connGroup").value || undefined,
    tags: document.getElementById("connTags").value.split(",").map(t => t.trim()).filter(Boolean),
  }
  if (data.authType === "password") data.password = document.getElementById("connPassword").value
  else data.privateKeyPath = document.getElementById("connKeyPath").value

  try {
    if (id) { await API.updateConnection(id, data); showToast(t("connectionUpdated"), "success") }
    else { await API.createConnection(data); showToast(t("connectionCreated"), "success") }
    closeModal("connectionModal"); await loadData()
  } catch (err) { showToast(err.message, "error") }
}

async function launchConnection(id) {
  try {
    const result = await API.launchConnection(id)
    showToast(result.message, result.success ? "info" : "error")
  } catch (err) { showToast(err.message, "error") }
}

async function deleteConnection(id) {
  showConfirm(t("confirmDelete"), async () => {
    try { await API.deleteConnection(id); showToast(t("connectionDeleted"), "success"); await loadData() }
    catch (err) { showToast(err.message, "error") }
  })
}

function showGroupModal(id) {
  document.getElementById("groupForm").reset()
  document.getElementById("groupId").value = ""
  document.getElementById("groupModalTitle").textContent = id ? t("editGroup") : t("newGroup")
  if (id) {
    const group = groups.find(g => g.id === id)
    if (!group) return
    document.getElementById("groupId").value = group.id
    document.getElementById("groupName").value = group.name
    document.getElementById("groupColor").value = group.color || "#9DD99A"
  }
  openModal("groupModal")
}

async function saveGroup(e) {
  e.preventDefault()
  const id = document.getElementById("groupId").value
  const data = { name: document.getElementById("groupName").value, color: document.getElementById("groupColor").value }
  try {
    if (id) { await API.updateGroup(id, data); showToast(t("groupUpdated"), "success") }
    else { await API.createGroup(data); showToast(t("groupCreated"), "success") }
    closeModal("groupModal"); await loadData()
  } catch (err) { showToast(err.message, "error") }
}

  async function deleteGroup(id) {
  const groupConns = connections.filter(c => c.groupId === id)
  if (groupConns.length > 0) { showToast(t("deleteGroupNotEmpty"), "error"); return }
  showConfirm(t("confirmDeleteGroup"), async () => {
    try { await API.deleteGroup(id); showToast(t("groupDeleted"), "success"); await loadData() }
    catch (err) { showToast(err.message, "error") }
  })
}

// ─── SETTINGS ────────────────────────────────────

function showSettingsModal() {
  document.getElementById("settingsTitle").textContent = t("settings")
  document.getElementById("settingsLang").value = settings.lang
  document.getElementById("settingsTheme").value = settings.theme
  document.getElementById("settingsUiScale").value = settings.uiScale
  document.getElementById("settingsStartWithSystem").checked = settings.startWithSystem
  document.getElementById("settingsStartMinimized").checked = settings.startMinimized
  document.getElementById("settingsCloseToTray").checked = settings.closeToTray
  openModal("settingsModal")
  loadTagManager()
}

async function loadTagManager() {
  const container = document.getElementById("tagManagerList")
  try {
    const tags = await API.listTags()
    if (tags.length === 0) {
      container.innerHTML = `<span style="color:var(--text-muted);font-size:12px">${t("noTags")}</span>`
      return
    }
    container.innerHTML = tags.map(t => `<div class="tag-manager-row">
      <span class="tag-manager-name">${esc(t.name)}</span>
      <span class="tag-manager-count">${t.count}</span>
      <button class="tag-manager-btn" data-tag="${esc(t.name)}" data-action="rename">${PENCIL_ICON}</button>
      <button class="tag-manager-btn danger" data-tag="${esc(t.name)}" data-action="delete">${TRASH_ICON}</button>
    </div>`).join("")
    container.querySelectorAll("[data-action='rename']").forEach(btn => {
      btn.addEventListener("click", async () => {
        const oldName = btn.dataset.tag
        const newName = prompt(t("renameTagPrompt"), oldName)
        if (!newName || newName === oldName) return
        try {
          await API.renameTag(oldName, newName)
          showToast(t("tagRenamed"), "success")
          await loadData()
          loadTagManager()
        } catch (err) { showToast(err.message, "error") }
      })
    })
    container.querySelectorAll("[data-action='delete']").forEach(btn => {
      btn.addEventListener("click", () => {
        const name = btn.dataset.tag
        showConfirm(t("confirmDeleteTag", { tag: name }), async () => {
          try {
            await API.deleteTag(name)
            showToast(t("tagDeleted"), "success")
            await loadData()
            loadTagManager()
          } catch (err) { showToast(err.message, "error") }
        })
      })
    })
  } catch (err) { container.innerHTML = `<span style="color:var(--text-muted);font-size:12px">${err.message}</span>` }
}

async function saveSettings() {
  const newSettings = {
    lang: document.getElementById("settingsLang").value,
    theme: document.getElementById("settingsTheme").value,
    uiScale: parseFloat(document.getElementById("settingsUiScale").value) || 1,
    startWithSystem: document.getElementById("settingsStartWithSystem").checked,
    startMinimized: document.getElementById("settingsStartMinimized").checked,
    closeToTray: document.getElementById("settingsCloseToTray").checked,
  }
  try {
    await API.saveSettings(newSettings)
    settings = newSettings
    applySettings()
    showToast(t("settings") + " saved", "success")
    closeModal("settingsModal")
    render()
  } catch (err) { showToast(err.message, "error") }
}

function downloadSampleImport() {
  const sample = {
    groups: [
      { name: "Serwery", color: "#9DD99A" },
      { name: "Bazy danych", color: "#5DA6EA" },
    ],
    connections: [
      { name: "Serwer WWW", host: "192.168.1.10", port: 22, username: "root", protocol: "ssh", authType: "password", tags: ["www", "prod"] },
      { name: "Baza MySQL", host: "db.example.com", port: 22, username: "admin", protocol: "ssh", authType: "key", privateKeyPath: "/home/user/.ssh/id_rsa", groupId: "Bazy danych" },
      { name: "Pulpit zdalny", host: "10.0.0.5", port: 3389, username: "admin", protocol: "rdp", authType: "password", groupId: "Serwery" },
    ],
  }
  const json = JSON.stringify(sample, null, 2)
  const blob = new Blob([json], { type: "application/json" })
  const url = URL.createObjectURL(blob)
  const a = document.createElement("a")
  a.href = url
  a.download = "rdesq-sample-import.json"
  a.click()
  URL.revokeObjectURL(url)
}

async function exportData() {
  try {
    const data = JSON.stringify(await API.exportData(), null, 2)
    await API.saveExportFile(data)
    showToast(t("export") + " OK", "success")
  } catch (err) {
    const msg = err?.message ?? err
    if (msg !== "cancelled") showToast(msg, "error")
  }
}

async function importData() {
  const input = document.getElementById("importFileInput")
  input.value = ""
  input.click()
}

document.getElementById("importFileInput").addEventListener("change", async (e) => {
  const file = e.target.files[0]
  if (!file) return
  try {
    const text = await file.text()
    const data = JSON.parse(text)
    if (!data.connections || !Array.isArray(data.connections)) {
      showToast(t("importInvalidFormat") || "Invalid import file format", "error")
      return
    }
    const result = await API.importData(data)
    showToast(t("importSuccess", { imported: result.imported, total: result.total }), "success")
    await loadData()
  } catch (err) { showToast((err?.message ?? err) || "Import failed", "error") }
})

// ─── QUICK CONNECT ───────────────────────────────

function showQuickConnect() {
  document.getElementById("quickConnectTitle").textContent = t("quickConnect")
  openModal("quickConnectModal")
}

async function saveQuickConnect(e) {
  e.preventDefault()
  const host = document.getElementById("qcHost").value.trim()
  if (!host) return
  let username = document.getElementById("qcUsername").value.trim() || "root"
  let port = parseInt(document.getElementById("qcPort").value) || 22
  let protocol = document.getElementById("qcProtocol").value
  const data = { name: host, host, port, username, protocol, authType: "password", password: "", tags: [] }
  try {
    const conn = await API.createConnection(data)
    showToast(t("connectionCreated"), "success")
    await loadData()
    closeModal("quickConnectModal")
  } catch (err) { showToast(err.message, "error") }
}

// ─── AUTH FORM TOGGLES ───────────────────────────

function toggleAuthFields() {
  const type = document.getElementById("connAuthType").value
  document.getElementById("passwordGroup").style.display = type === "password" ? "block" : "none"
  document.getElementById("keyGroup").style.display = type === "key" ? "block" : "none"
}

function togglePort() {
  const proto = document.getElementById("connProtocol").value
  const portInput = document.getElementById("connPort")
  if (!portInput.dataset.userChanged) portInput.value = proto === "ssh" ? 22 : 3389
}

// ─── KEYBOARD SHORTCUTS ──────────────────────────

document.addEventListener("keydown", (e) => {
  if (e.ctrlKey || e.metaKey) {
    switch (e.key) {
      case "q": e.preventDefault(); showQuickConnect(); break
      case "n": e.preventDefault(); showConnectionModal(); break
      case "f": e.preventDefault(); document.getElementById("searchInput").focus(); break
      case "s": e.preventDefault(); if (document.getElementById("settingsModal").style.display === "flex") saveSettings(); break
    }
  }
  if (e.key === "Escape") {
    document.querySelectorAll(".modal-overlay[style*='flex']").forEach(m => m.style.display = "none")
  }
})

// ─── TOAST ───────────────────────────────────────

function showToast(message, type = "info") {
  const container = document.getElementById("toastContainer")
  const toast = document.createElement("div")
  toast.className = `toast toast-${type}`
  toast.textContent = message
  container.appendChild(toast)
  setTimeout(() => { toast.style.opacity = "0"; setTimeout(() => toast.remove(), 300) }, 3000)
}

// ─── INIT ────────────────────────────────────────

function init() {
  document.getElementById("connectionForm").addEventListener("submit", saveConnection)
  document.getElementById("groupForm").addEventListener("submit", saveGroup)

  document.getElementById("searchInput").addEventListener("input", filterConnections)
  document.getElementById("sortDirBtn").addEventListener("click", toggleSortDir)
  document.getElementById("sortFieldTrigger").addEventListener("click", () => {
    document.getElementById("sortDropdownMenu").classList.toggle("open")
  })
  document.querySelectorAll(".sort-dropdown-option").forEach(el => {
    el.addEventListener("click", () => onSortFieldChange(el.dataset.value))
  })
  document.getElementById("settingsBtn").addEventListener("click", showSettingsModal)
  document.getElementById("pingAllBtn").addEventListener("click", pingAllHosts)
  document.getElementById("addConnBtn").addEventListener("click", () => showConnectionModal())
  document.getElementById("addGroupBtn").addEventListener("click", () => showGroupModal())
  document.getElementById("quickConnectBtn").addEventListener("click", showQuickConnect)
  document.getElementById("saveSettingsBtn").addEventListener("click", saveSettings)
  document.getElementById("exportBtn").addEventListener("click", exportData)
  document.getElementById("importBtn").addEventListener("click", importData)
  document.getElementById("sampleBtn").addEventListener("click", downloadSampleImport)
  document.getElementById("qcForm").addEventListener("submit", saveQuickConnect)

  // toggle password visibility
  const pwInput = document.getElementById("connPassword")
  const pwToggle = document.getElementById("togglePassword")
  if (pwToggle && pwInput) {
    const updateEye = () => { pwToggle.innerHTML = pwInput.type === "password" ? EYE_ICON : EYE_OFF_ICON }
    updateEye()
    pwToggle.addEventListener("click", () => {
      pwInput.type = pwInput.type === "password" ? "text" : "password"
      updateEye()
    })
  }

  try {
    const api = window.__TAURI__
    const wmod = api?.window
    const gcw = wmod?.getCurrentWindow
    let win

    if (gcw) {
      win = gcw()
      if (win && typeof win.minimize !== 'function') win = null
    }

    if (win) {
      appWin = win

      document.getElementById('minBtn').onclick = () => { win.minimize().catch(() => {}) }
      document.getElementById('maxBtn').onclick = () => {
        win.isMaximized().then(max => {
          if (max) { win.unmaximize().catch(() => {}) } else { win.maximize().catch(() => {}) }
        }).catch(() => {})
      }
      document.getElementById('closeBtn').onclick = () => { win.close().catch(() => {}) }
      document.querySelector('.header').addEventListener('mousedown', (e) => {
        if (e.target.closest('button,input,select,textarea,.titlebar-controls')) return
        win.startDragging().catch(() => {})
      })

      // ── window state persistence ──
      const K = 'rdesq_win'
      let lastState = null

      const saveState = () => {
        Promise.all([win.outerSize(), win.outerPosition(), win.isMaximized()])
          .then(([sz, pos, max]) => {
            lastState = { x: pos.x, y: pos.y, w: sz.width, h: sz.height, max }
            try { localStorage.setItem(K, JSON.stringify(lastState)) } catch (_) {}
          }).catch(() => {})
      }

      let rsTimer
      window.addEventListener('resize', () => { clearTimeout(rsTimer); rsTimer = setTimeout(saveState, 300) })
      setInterval(saveState, 3000)

      const closeBtn = document.getElementById('closeBtn')
      closeBtn.onclick = () => {
        if (lastState) { try { localStorage.setItem(K, JSON.stringify(lastState)) } catch (_) {} }
        win.close().catch(() => {})
      }

      const restoreState = () => {
        try {
          const raw = localStorage.getItem(K)
          if (!raw) return
          const s = JSON.parse(raw)
          if (typeof s.w !== 'number' || s.w < 100 || s.h < 100) return
          const PS = window.__TAURI__.window.PhysicalSize
          const PP = window.__TAURI__.window.PhysicalPosition
          if (typeof PS !== 'function' || typeof PP !== 'function') return
          const ops = [win.setSize(new PS(s.w, s.h))]
          if (s.x != null && s.y != null) ops.push(win.setPosition(new PP(s.x, s.y)))
          Promise.all(ops).then(() => {
            if (s.max) return win.maximize()
          }).catch(() => {})
        } catch (_) {}
      }
      restoreState()
    }
  } catch(_) {}

  document.getElementById("connAuthType").addEventListener("change", toggleAuthFields)
  document.getElementById("connProtocol").addEventListener("change", function () {
    const portInput = document.getElementById("connPort")
    portInput.dataset.userChanged = ""
    portInput.value = this.value === "ssh" ? 22 : 3389
  })
  document.getElementById("connPort").addEventListener("input", function () { this.dataset.userChanged = "true" })

  loadData()
  initTooltips()
}

init()

// ─── CUSTOM TOOLTIPS ──────────────────────────────

function initTooltips() {
  const tip = document.getElementById("tooltip")
  let timer

  function show(e) {
    const el = e.target.closest("[data-tooltip]")
    const text = el?.dataset.tooltip
    if (!text) return
    tip.textContent = text
    tip.classList.add("visible")

    const rect = el.getBoundingClientRect()
    const tipW = tip.offsetWidth
    const tipH = tip.offsetHeight
    const gap = 6
    const rightX = rect.right + gap + tipW
    const leftX = rect.left - gap - tipW

    let top, left
    if (rightX <= window.innerWidth) {
      top = rect.top + (rect.height - tipH) / 2
      left = rect.right + gap
    } else if (leftX >= 0) {
      top = rect.top + (rect.height - tipH) / 2
      left = rect.left - gap - tipW
    } else {
      top = rect.bottom + gap
      left = rect.left + (rect.width - tipW) / 2
    }

    tip.style.top = Math.max(4, Math.min(top, window.innerHeight - tipH - 4)) + "px"
    tip.style.left = Math.max(4, left) + "px"
  }

  function hide() {
    tip.classList.remove("visible")
  }

  document.addEventListener("mouseover", (e) => {
    const el = e.target.closest("[data-tooltip]")
    if (!el) { hide(); return }
    clearTimeout(timer)
    timer = setTimeout(() => show(e), 400)
  })

  document.addEventListener("mouseout", (e) => {
    if (e.target.closest("[data-tooltip]")) {
      clearTimeout(timer); hide()
    }
  })
}
