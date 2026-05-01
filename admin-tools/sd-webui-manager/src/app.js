const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

const elements = {
    statusBadge: document.getElementById("status-badge"),
    btnStart: document.getElementById("btn-start"),
    btnStop: document.getElementById("btn-stop"),
    btnRestart: document.getElementById("btn-restart"),
    btnSettings: document.getElementById("btn-settings"),
    btnCloseSettings: document.getElementById("btn-close-settings"),
    btnSaveSettings: document.getElementById("btn-save-settings"),
    btnCancelSettings: document.getElementById("btn-cancel-settings"),
    btnDetectPath: document.getElementById("btn-detect-path"),
    btnClearLogs: document.getElementById("btn-clear-logs"),
    logOutput: document.getElementById("log-output"),
    logAutoscroll: document.getElementById("log-autoscroll"),
    settingsModal: document.getElementById("settings-modal"),
    infoPid: document.getElementById("info-pid"),
    infoHealth: document.getElementById("info-health"),
    infoApiPort: document.getElementById("info-api-port"),
    settingSdPath: document.getElementById("setting-sd-path"),
    settingSdPort: document.getElementById("setting-sd-port"),
    settingApiPort: document.getElementById("setting-api-port"),
    settingArgs: document.getElementById("setting-args"),
    settingAutoStart: document.getElementById("setting-auto-start"),
};

let currentStatus = "stopped";
let healthCheckInterval = null;

function updateStatusUI(status) {
    currentStatus = status.split(":")[0];
    const statusText = status.includes(":") ? status.split(":").slice(1).join(":") : status;

    elements.statusBadge.textContent = statusText.charAt(0).toUpperCase() + statusText.slice(1);
    elements.statusBadge.className = "badge badge-" + currentStatus;

    const isRunning = currentStatus === "running";
    const isStopped = currentStatus === "stopped" || currentStatus === "error";
    const isTransitioning = currentStatus === "starting" || currentStatus === "stopping";

    elements.btnStart.disabled = !isStopped;
    elements.btnStop.disabled = isStopped || isTransitioning;
    elements.btnRestart.disabled = isStopped || isTransitioning;

    if (isRunning) {
        startHealthCheck();
    } else {
        stopHealthCheck();
        elements.infoPid.textContent = "-";
        elements.infoHealth.textContent = "-";
        elements.infoHealth.style.color = "";
    }
}

function appendLog(line) {
    const div = document.createElement("div");
    div.className = "log-line";

    if (line.includes("[ERROR]") || line.toLowerCase().includes("error:") || line.includes("Traceback")) {
        div.classList.add("log-error");
    } else if (line.includes("[WARNING]") || line.toLowerCase().includes("warn:")) {
        div.classList.add("log-warn");
    } else if (line.includes("[INFO]") || line.includes("Running on")) {
        div.classList.add("log-info");
    } else if (line.includes("[Manager]")) {
        div.classList.add("log-manager");
    }

    div.textContent = line;
    elements.logOutput.appendChild(div);

    if (elements.logOutput.children.length > 10000) {
        elements.logOutput.removeChild(elements.logOutput.firstChild);
    }

    if (elements.logAutoscroll.checked) {
        elements.logOutput.scrollTop = elements.logOutput.scrollHeight;
    }
}

async function startHealthCheck() {
    if (healthCheckInterval) return;
    healthCheckInterval = setInterval(async () => {
        try {
            const health = await invoke("check_sd_server_health");
            elements.infoHealth.textContent = health;
            elements.infoHealth.style.color = health === "healthy" ? "var(--accent-green)" : "var(--accent-yellow)";
        } catch {
            elements.infoHealth.textContent = "check failed";
            elements.infoHealth.style.color = "var(--accent-red)";
        }
    }, 10000);
}

function stopHealthCheck() {
    if (healthCheckInterval) {
        clearInterval(healthCheckInterval);
        healthCheckInterval = null;
    }
}

async function loadConfig() {
    try {
        const config = await invoke("get_config");
        elements.settingSdPath.value = config.sd_webui_path || "";
        elements.settingSdPort.value = config.sd_port || 7860;
        elements.settingApiPort.value = config.api_port || 9786;
        elements.settingArgs.value = config.commandline_args || "";
        elements.settingAutoStart.checked = config.auto_start || false;
        elements.infoApiPort.textContent = config.api_port || 9786;
    } catch (e) {
        appendLog("[Manager] Failed to load config: " + e);
    }
}

async function saveConfig() {
    const config = {
        sd_webui_path: elements.settingSdPath.value || null,
        sd_port: parseInt(elements.settingSdPort.value) || 7860,
        api_port: parseInt(elements.settingApiPort.value) || 9786,
        commandline_args: elements.settingArgs.value || null,
        auto_start: elements.settingAutoStart.checked,
    };

    try {
        await invoke("save_config", { config });
        elements.infoApiPort.textContent = config.api_port;
        elements.settingsModal.classList.add("hidden");
        appendLog("[Manager] Configuration saved");
    } catch (e) {
        appendLog("[Manager] Failed to save config: " + e);
    }
}

elements.btnStart.addEventListener("click", async () => {
    try {
        elements.btnStart.disabled = true;
        const result = await invoke("start_server");
        appendLog("[Manager] " + result);
    } catch (e) {
        appendLog("[Manager] Error: " + e);
    }
});

elements.btnStop.addEventListener("click", async () => {
    try {
        elements.btnStop.disabled = true;
        const result = await invoke("stop_server");
        appendLog("[Manager] " + result);
    } catch (e) {
        appendLog("[Manager] Error: " + e);
    }
});

elements.btnRestart.addEventListener("click", async () => {
    try {
        elements.btnRestart.disabled = true;
        const result = await invoke("restart_server");
        appendLog("[Manager] " + result);
    } catch (e) {
        appendLog("[Manager] Error: " + e);
    }
});

elements.btnSettings.addEventListener("click", () => {
    loadConfig();
    elements.settingsModal.classList.remove("hidden");
});

elements.btnCloseSettings.addEventListener("click", () => {
    elements.settingsModal.classList.add("hidden");
});

elements.btnCancelSettings.addEventListener("click", () => {
    elements.settingsModal.classList.add("hidden");
});

elements.btnSaveSettings.addEventListener("click", saveConfig);

elements.btnDetectPath.addEventListener("click", async () => {
    try {
        const path = await invoke("detect_sd_webui_path");
        if (path) {
            elements.settingSdPath.value = path;
        } else {
            appendLog("[Manager] Could not auto-detect SD WebUI path");
        }
    } catch (e) {
        appendLog("[Manager] Detection error: " + e);
    }
});

elements.btnClearLogs.addEventListener("click", () => {
    elements.logOutput.innerHTML = "";
});

listen("server-log", (event) => {
    appendLog(event.payload);
});

listen("server-status", (event) => {
    updateStatusUI(event.payload);
});

async function init() {
    try {
        const status = await invoke("get_status");
        updateStatusUI(status);

        const logs = await invoke("get_logs", { lastN: 100 });
        for (const line of logs) {
            appendLog(line);
        }

        const config = await invoke("get_config");
        elements.infoApiPort.textContent = config.api_port || 9786;

        if (config.auto_start && (status === "stopped" || status === "error")) {
            appendLog("[Manager] Auto-start enabled, starting server...");
            invoke("start_server");
        }
    } catch (e) {
        appendLog("[Manager] Init error: " + e);
    }
}

init();
