<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";

// ========== 类型定义 ==========
interface CpuInfo {
  name: string;
  usage: number;
  cores: number;
  core_usages: number[];
}
interface MemoryInfo {
  total_gb: number;
  used_gb: number;
  usage_percent: number;
}
interface DiskInfo {
  name: string;
  mount_point: string;
  total_gb: number;
  used_gb: number;
  available_gb: number;
  usage_percent: number;
  fs_type: string;
}
interface NetworkInterface {
  name: string;
  received_bytes: number;
  transmitted_bytes: number;
  mac_address: string;
  ip_addresses: string[];
}
interface TempInfo {
  label: string;
  temperature: number;
}
interface ProcessInfo {
  name: string;
  pid: number;
  cpu_usage: number;
  memory_mb: number;
}
interface FullSystemInfo {
  overview: { os_name: string; host_name: string; uptime: number };
  cpu: CpuInfo;
  memory: MemoryInfo;
  disks: DiskInfo[];
  networks: NetworkInterface[];
  temperatures: TempInfo[];
  top_processes: ProcessInfo[];
}
interface PublicIpInfo {
  ip: string;
  city: string | null;
  region: string | null;
  country: string | null;
  org: string | null;
}
interface BatteryInfo {
  percentage: number;
  is_charging: boolean;
  state: string;
  health_percent: number;
  time_to_empty_mins: number | null;
  time_to_full_mins: number | null;
  cycle_count: number | null;
}

// ========== 状态 ==========
const currentPage = ref("overview");
const info = ref<FullSystemInfo | null>(null);
const publicIp = ref<PublicIpInfo | null>(null);
const batteryInfo = ref<BatteryInfo | null>(null);
const loading = ref(true);
const ipLoading = ref(true);

// 历史记录（最近60个点，约90秒）
const cpuHistory = reactive<number[]>([]);
const memHistory = reactive<number[]>([]);
const MAX_HISTORY = 60;

// 网络速度计算
const prevNetBytes = ref<{ received: number; transmitted: number } | null>(null);
const netSpeed = ref({ download: 0, upload: 0 });

let timer: number;

// ========== 页面导航 ==========
const pages = [
  { id: "overview", icon: "📊", label: "概览" },
  { id: "cpu", icon: "🔥", label: "CPU" },
  { id: "memory", icon: "💾", label: "内存" },
  { id: "disk", icon: "💿", label: "磁盘" },
  { id: "network", icon: "🌐", label: "网络" },
  { id: "process", icon: "📋", label: "进程" },
  { id: "battery", icon: "🔋", label: "电池" },
];

// ========== 工具函数 ==========
function usageColor(usage: number): string {
  if (usage < 40) return "#4ade80";
  if (usage < 70) return "#facc15";
  return "#ef4444";
}

function formatUptime(seconds: number): string {
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  if (d > 0) return `${d}天 ${h}小时 ${m}分钟`;
  if (h > 0) return `${h}小时 ${m}分钟`;
  return `${m}分钟`;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return bytes + " B";
  if (bytes < 1048576) return (bytes / 1024).toFixed(1) + " KB";
  if (bytes < 1073741824) return (bytes / 1048576).toFixed(1) + " MB";
  return (bytes / 1073741824).toFixed(2) + " GB";
}

function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec < 1024) return bytesPerSec.toFixed(0) + " B/s";
  if (bytesPerSec < 1048576) return (bytesPerSec / 1024).toFixed(1) + " KB/s";
  return (bytesPerSec / 1048576).toFixed(1) + " MB/s";
}

// SVG 折线图路径
function historyToPath(history: number[], width: number, height: number): string {
  if (history.length < 2) return "";
  const step = width / (MAX_HISTORY - 1);
  return history
    .map((v, i) => {
      const x = i * step;
      const y = height - (v / 100) * height;
      return `${i === 0 ? "M" : "L"} ${x.toFixed(1)} ${y.toFixed(1)}`;
    })
    .join(" ");
}

function historyToArea(history: number[], width: number, height: number): string {
  if (history.length < 2) return "";
  const path = historyToPath(history, width, height);
  const lastX = ((history.length - 1) * width) / (MAX_HISTORY - 1);
  return `${path} L ${lastX.toFixed(1)} ${height} L 0 ${height} Z`;
}

// ========== 数据获取 ==========
async function fetchInfo() {
  try {
    const data = await invoke<FullSystemInfo>("get_system_info");
    info.value = data;
    loading.value = false;

    // 更新历史
    cpuHistory.push(data.cpu.usage);
    if (cpuHistory.length > MAX_HISTORY) cpuHistory.shift();
    memHistory.push(data.memory.usage_percent);
    if (memHistory.length > MAX_HISTORY) memHistory.shift();

    // 计算网络速度
    const totalReceived = data.networks.reduce((s, n) => s + n.received_bytes, 0);
    const totalTransmitted = data.networks.reduce((s, n) => s + n.transmitted_bytes, 0);
    if (prevNetBytes.value) {
      const dt = 1.5; // 刷新间隔秒数
      netSpeed.value = {
        download: Math.max(0, (totalReceived - prevNetBytes.value.received) / dt),
        upload: Math.max(0, (totalTransmitted - prevNetBytes.value.transmitted) / dt),
      };
    }
    prevNetBytes.value = { received: totalReceived, transmitted: totalTransmitted };
  } catch (e) {
    console.error("获取系统信息失败:", e);
  }
}

async function fetchPublicIp() {
  try {
    publicIp.value = await invoke<PublicIpInfo>("get_public_ip");
  } catch (e) {
    console.error("获取公网IP失败:", e);
  } finally {
    ipLoading.value = false;
  }
}

async function fetchBattery() {
  try {
    batteryInfo.value = await invoke<BatteryInfo | null>("get_battery_info");
  } catch (e) {
    console.error("获取电池信息失败:", e);
  }
}

// ========== 生命周期 ==========
onMounted(() => {
  fetchInfo();
  fetchPublicIp();
  fetchBattery();
  timer = window.setInterval(() => {
    fetchInfo();
    fetchBattery();
  }, 1500);
});

onUnmounted(() => clearInterval(timer));

// ========== 计算属性 ==========
const activeNetworks = computed(() => {
  if (!info.value) return [];
  return info.value.networks.filter(
    (n) => n.received_bytes > 0 || n.transmitted_bytes > 0
  );
});
</script>

<template>
  <div class="app-layout" v-if="!loading && info">
    <!-- 侧边栏 -->
    <aside class="sidebar">
      <div class="sidebar-title">⚡ 仪表盘</div>
      <nav class="nav-list">
        <button
          v-for="page in pages"
          :key="page.id"
          :class="['nav-item', { active: currentPage === page.id }]"
          @click="currentPage = page.id"
        >
          <span class="nav-icon">{{ page.icon }}</span>
          <span class="nav-label">{{ page.label }}</span>
        </button>
      </nav>
      <div class="sidebar-footer">
        <div class="host-name">{{ info.overview.host_name }}</div>
        <div class="os-name">{{ info.overview.os_name }}</div>
      </div>
    </aside>

    <!-- 内容区域 -->
    <main class="content">
      <!-- ==================== 概览页 ==================== -->
      <div v-if="currentPage === 'overview'" class="page">
        <h1 class="page-title">系统概览</h1>
        <div class="overview-grid">
          <!-- CPU 卡片 -->
          <div class="card glass" @click="currentPage = 'cpu'" style="cursor: pointer">
            <div class="card-header">
              <span class="card-icon">🔥</span>
              <span class="card-label">CPU</span>
              <span class="chip" :style="{ background: usageColor(info.cpu.usage) }">
                {{ info.cpu.usage.toFixed(1) }}%
              </span>
            </div>
            <p class="card-sub">{{ info.cpu.name }}</p>
            <div class="bar-bg"><div class="bar-fill" :style="{ width: info.cpu.usage + '%', background: usageColor(info.cpu.usage) }"></div></div>
            <!-- 迷你折线图 -->
            <svg class="mini-chart" viewBox="0 0 200 40" preserveAspectRatio="none">
              <path :d="historyToArea(cpuHistory, 200, 40)" fill="rgba(74, 222, 128, 0.15)" />
              <path :d="historyToPath(cpuHistory, 200, 40)" fill="none" stroke="#4ade80" stroke-width="1.5" />
            </svg>
          </div>

          <!-- 内存卡片 -->
          <div class="card glass" @click="currentPage = 'memory'" style="cursor: pointer">
            <div class="card-header">
              <span class="card-icon">💾</span>
              <span class="card-label">内存</span>
              <span class="chip" :style="{ background: usageColor(info.memory.usage_percent) }">
                {{ info.memory.usage_percent.toFixed(1) }}%
              </span>
            </div>
            <p class="card-sub">{{ info.memory.used_gb.toFixed(1) }} / {{ info.memory.total_gb.toFixed(1) }} GB</p>
            <div class="bar-bg"><div class="bar-fill" :style="{ width: info.memory.usage_percent + '%', background: usageColor(info.memory.usage_percent) }"></div></div>
            <svg class="mini-chart" viewBox="0 0 200 40" preserveAspectRatio="none">
              <path :d="historyToArea(memHistory, 200, 40)" fill="rgba(96, 165, 250, 0.15)" />
              <path :d="historyToPath(memHistory, 200, 40)" fill="none" stroke="#60a5fa" stroke-width="1.5" />
            </svg>
          </div>

          <!-- 磁盘卡片 -->
          <div class="card glass" @click="currentPage = 'disk'" style="cursor: pointer">
            <div class="card-header">
              <span class="card-icon">💿</span>
              <span class="card-label">磁盘</span>
            </div>
            <div v-for="disk in info.disks.slice(0, 2)" :key="disk.mount_point" class="disk-mini">
              <div class="disk-mini-header">
                <span>{{ disk.mount_point }}</span>
                <span>{{ disk.usage_percent.toFixed(0) }}%</span>
              </div>
              <div class="bar-bg bar-sm"><div class="bar-fill" :style="{ width: disk.usage_percent + '%', background: usageColor(disk.usage_percent) }"></div></div>
            </div>
          </div>

          <!-- 网络卡片 -->
          <div class="card glass" @click="currentPage = 'network'" style="cursor: pointer">
            <div class="card-header">
              <span class="card-icon">🌐</span>
              <span class="card-label">网络</span>
            </div>
            <div class="net-mini">
              <div class="net-mini-row">
                <span class="net-arrow down">↓</span>
                <span>{{ formatSpeed(netSpeed.download) }}</span>
              </div>
              <div class="net-mini-row">
                <span class="net-arrow up">↑</span>
                <span>{{ formatSpeed(netSpeed.upload) }}</span>
              </div>
            </div>
            <p class="card-sub" v-if="publicIp">{{ publicIp.ip }} · {{ publicIp.country }}</p>
            <p class="card-sub" v-else-if="ipLoading">获取IP中...</p>
          </div>

          <!-- 运行时间卡片 -->
          <div class="card glass">
            <div class="card-header">
              <span class="card-icon">⏱️</span>
              <span class="card-label">运行时间</span>
            </div>
            <p class="uptime-text">{{ formatUptime(info.overview.uptime) }}</p>
          </div>

          <!-- 电池卡片 -->
          <div class="card glass" @click="currentPage = 'battery'" style="cursor: pointer" v-if="batteryInfo">
            <div class="card-header">
              <span class="card-icon">🔋</span>
              <span class="card-label">电池</span>
              <span class="chip" :style="{ background: usageColor(100 - batteryInfo.percentage) }">
                {{ batteryInfo.percentage.toFixed(0) }}%
              </span>
            </div>
            <p class="card-sub">{{ batteryInfo.state }}</p>
            <div class="bar-bg"><div class="bar-fill" :style="{ width: batteryInfo.percentage + '%', background: usageColor(100 - batteryInfo.percentage) }"></div></div>
          </div>
        </div>
      </div>

      <!-- ==================== CPU 页 ==================== -->
      <div v-if="currentPage === 'cpu'" class="page">
        <h1 class="page-title">🔥 CPU 详情</h1>
        <div class="card glass">
          <div class="card-header">
            <span class="card-label">{{ info.cpu.name }}</span>
            <span class="chip" :style="{ background: usageColor(info.cpu.usage) }">
              {{ info.cpu.usage.toFixed(1) }}%
            </span>
          </div>
          <p class="card-sub">{{ info.cpu.cores }} 核心</p>
          <div class="bar-bg"><div class="bar-fill" :style="{ width: info.cpu.usage + '%', background: usageColor(info.cpu.usage) }"></div></div>
        </div>

        <!-- CPU 历史折线图 -->
        <div class="card glass">
          <h3 class="section-title">使用率历史 (最近 90 秒)</h3>
          <div class="chart-container">
            <svg class="chart" viewBox="0 0 500 150" preserveAspectRatio="none">
              <!-- 网格线 -->
              <line x1="0" y1="37.5" x2="500" y2="37.5" stroke="#333" stroke-width="0.5" />
              <line x1="0" y1="75" x2="500" y2="75" stroke="#333" stroke-width="0.5" />
              <line x1="0" y1="112.5" x2="500" y2="112.5" stroke="#333" stroke-width="0.5" />
              <!-- 数据 -->
              <path :d="historyToArea(cpuHistory, 500, 150)" fill="url(#cpuGrad)" />
              <path :d="historyToPath(cpuHistory, 500, 150)" fill="none" stroke="#4ade80" stroke-width="2" />
              <defs>
                <linearGradient id="cpuGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="rgba(74, 222, 128, 0.3)" />
                  <stop offset="100%" stop-color="rgba(74, 222, 128, 0)" />
                </linearGradient>
              </defs>
            </svg>
            <div class="chart-labels">
              <span>100%</span><span>75%</span><span>50%</span><span>25%</span><span>0%</span>
            </div>
          </div>
        </div>

        <!-- 各核心使用率 -->
        <div class="card glass">
          <h3 class="section-title">各核心使用率</h3>
          <div class="cores-grid">
            <div v-for="(usage, i) in info.cpu.core_usages" :key="i" class="core-item">
              <div class="core-bar-bg">
                <div class="core-bar-fill" :style="{ height: usage + '%', background: usageColor(usage) }"></div>
              </div>
              <div class="core-label">{{ i }}</div>
              <div class="core-value">{{ usage.toFixed(0) }}%</div>
            </div>
          </div>
        </div>
      </div>

      <!-- ==================== 内存页 ==================== -->
      <div v-if="currentPage === 'memory'" class="page">
        <h1 class="page-title">💾 内存详情</h1>
        <div class="card glass">
          <div class="card-header">
            <span class="card-label">内存使用</span>
            <span class="chip" :style="{ background: usageColor(info.memory.usage_percent) }">
              {{ info.memory.usage_percent.toFixed(1) }}%
            </span>
          </div>
          <div class="mem-stats">
            <div class="mem-stat">
              <span class="mem-stat-label">已使用</span>
              <span class="mem-stat-value">{{ info.memory.used_gb.toFixed(1) }} GB</span>
            </div>
            <div class="mem-stat">
              <span class="mem-stat-label">总计</span>
              <span class="mem-stat-value">{{ info.memory.total_gb.toFixed(1) }} GB</span>
            </div>
            <div class="mem-stat">
              <span class="mem-stat-label">可用</span>
              <span class="mem-stat-value">{{ (info.memory.total_gb - info.memory.used_gb).toFixed(1) }} GB</span>
            </div>
          </div>
          <div class="bar-bg bar-lg"><div class="bar-fill" :style="{ width: info.memory.usage_percent + '%', background: usageColor(info.memory.usage_percent) }"></div></div>
        </div>

        <div class="card glass">
          <h3 class="section-title">使用率历史 (最近 90 秒)</h3>
          <div class="chart-container">
            <svg class="chart" viewBox="0 0 500 150" preserveAspectRatio="none">
              <line x1="0" y1="37.5" x2="500" y2="37.5" stroke="#333" stroke-width="0.5" />
              <line x1="0" y1="75" x2="500" y2="75" stroke="#333" stroke-width="0.5" />
              <line x1="0" y1="112.5" x2="500" y2="112.5" stroke="#333" stroke-width="0.5" />
              <path :d="historyToArea(memHistory, 500, 150)" fill="url(#memGrad)" />
              <path :d="historyToPath(memHistory, 500, 150)" fill="none" stroke="#60a5fa" stroke-width="2" />
              <defs>
                <linearGradient id="memGrad" x1="0" y1="0" x2="0" y2="1">
                  <stop offset="0%" stop-color="rgba(96, 165, 250, 0.3)" />
                  <stop offset="100%" stop-color="rgba(96, 165, 250, 0)" />
                </linearGradient>
              </defs>
            </svg>
            <div class="chart-labels">
              <span>100%</span><span>75%</span><span>50%</span><span>25%</span><span>0%</span>
            </div>
          </div>
        </div>
      </div>

      <!-- ==================== 磁盘页 ==================== -->
      <div v-if="currentPage === 'disk'" class="page">
        <h1 class="page-title">💿 磁盘信息</h1>
        <div v-for="disk in info.disks" :key="disk.mount_point" class="card glass">
          <div class="card-header">
            <span class="card-label">{{ disk.name || disk.mount_point }}</span>
            <span class="chip" :style="{ background: usageColor(disk.usage_percent) }">
              {{ disk.usage_percent.toFixed(1) }}%
            </span>
          </div>
          <p class="card-sub">挂载点: {{ disk.mount_point }} · 文件系统: {{ disk.fs_type }}</p>
          <div class="mem-stats">
            <div class="mem-stat">
              <span class="mem-stat-label">已使用</span>
              <span class="mem-stat-value">{{ disk.used_gb.toFixed(1) }} GB</span>
            </div>
            <div class="mem-stat">
              <span class="mem-stat-label">可用</span>
              <span class="mem-stat-value">{{ disk.available_gb.toFixed(1) }} GB</span>
            </div>
            <div class="mem-stat">
              <span class="mem-stat-label">总计</span>
              <span class="mem-stat-value">{{ disk.total_gb.toFixed(1) }} GB</span>
            </div>
          </div>
          <div class="bar-bg"><div class="bar-fill" :style="{ width: disk.usage_percent + '%', background: usageColor(disk.usage_percent) }"></div></div>
        </div>
      </div>

      <!-- ==================== 网络页 ==================== -->
      <div v-if="currentPage === 'network'" class="page">
        <h1 class="page-title">🌐 网络信息</h1>

        <!-- 公网 IP -->
        <div class="card glass highlight-card">
          <div class="card-header">
            <span class="card-label">🌍 公网 IP</span>
          </div>
          <div v-if="publicIp" class="ip-info">
            <div class="ip-address">{{ publicIp.ip }}</div>
            <div class="ip-details">
              <span v-if="publicIp.city">{{ publicIp.city }}, </span>
              <span v-if="publicIp.region">{{ publicIp.region }}, </span>
              <span v-if="publicIp.country">{{ publicIp.country }}</span>
            </div>
            <div class="ip-org" v-if="publicIp.org">{{ publicIp.org }}</div>
          </div>
          <div v-else-if="ipLoading" class="ip-loading">获取中...</div>
          <div v-else class="ip-loading">获取失败</div>
          <button class="refresh-btn" @click="ipLoading = true; fetchPublicIp()">刷新 IP</button>
        </div>

        <!-- 实时速度 -->
        <div class="card glass">
          <div class="card-header">
            <span class="card-label">📶 实时速度</span>
          </div>
          <div class="speed-display">
            <div class="speed-item">
              <span class="speed-arrow down">↓</span>
              <div>
                <div class="speed-value">{{ formatSpeed(netSpeed.download) }}</div>
                <div class="speed-label">下载</div>
              </div>
            </div>
            <div class="speed-divider"></div>
            <div class="speed-item">
              <span class="speed-arrow up">↑</span>
              <div>
                <div class="speed-value">{{ formatSpeed(netSpeed.upload) }}</div>
                <div class="speed-label">上传</div>
              </div>
            </div>
          </div>
        </div>

        <!-- 网络接口 -->
        <div v-for="net in activeNetworks" :key="net.name" class="card glass">
          <div class="card-header">
            <span class="card-label">{{ net.name }}</span>
            <span class="mac-addr">{{ net.mac_address }}</span>
          </div>
          <div class="net-details">
            <div v-if="net.ip_addresses.length" class="net-ips">
              <span class="net-detail-label">IP 地址:</span>
              <span v-for="ip in net.ip_addresses" :key="ip" class="ip-tag">{{ ip }}</span>
            </div>
            <div class="net-traffic">
              <span>↓ {{ formatBytes(net.received_bytes) }}</span>
              <span>↑ {{ formatBytes(net.transmitted_bytes) }}</span>
            </div>
          </div>
        </div>
      </div>

      <!-- ==================== 进程页 ==================== -->
      <div v-if="currentPage === 'process'" class="page">
        <h1 class="page-title">📋 进程 (Top 10)</h1>
        <div class="card glass">
          <table class="process-table">
            <thead>
              <tr>
                <th class="th-name">名称</th>
                <th>PID</th>
                <th>CPU</th>
                <th>内存</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="proc in info.top_processes" :key="proc.pid">
                <td class="td-name">{{ proc.name }}</td>
                <td class="td-center">{{ proc.pid }}</td>
                <td class="td-center">
                  <span :style="{ color: usageColor(proc.cpu_usage) }">{{ proc.cpu_usage.toFixed(1) }}%</span>
                </td>
                <td class="td-center">{{ proc.memory_mb.toFixed(0) }} MB</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- ==================== 电池页 ==================== -->
      <div v-if="currentPage === 'battery'" class="page">
        <h1 class="page-title">🔋 电池信息</h1>
        <div v-if="batteryInfo" class="card glass">
          <div class="battery-visual">
            <div class="battery-icon-large">
              <div class="battery-body">
                <div class="battery-level" :style="{ width: batteryInfo.percentage + '%', background: usageColor(100 - batteryInfo.percentage) }"></div>
              </div>
              <div class="battery-tip"></div>
            </div>
            <div class="battery-percent">{{ batteryInfo.percentage.toFixed(0) }}%</div>
            <div class="battery-state">{{ batteryInfo.state }}</div>
          </div>
          <div class="mem-stats" style="margin-top: 20px">
            <div class="mem-stat">
              <span class="mem-stat-label">健康度</span>
              <span class="mem-stat-value">{{ batteryInfo.health_percent.toFixed(1) }}%</span>
            </div>
            <div class="mem-stat" v-if="batteryInfo.cycle_count">
              <span class="mem-stat-label">循环次数</span>
              <span class="mem-stat-value">{{ batteryInfo.cycle_count }}</span>
            </div>
            <div class="mem-stat" v-if="batteryInfo.time_to_empty_mins && !batteryInfo.is_charging">
              <span class="mem-stat-label">剩余时间</span>
              <span class="mem-stat-value">{{ Math.floor(batteryInfo.time_to_empty_mins / 60) }}h {{ Math.floor(batteryInfo.time_to_empty_mins % 60) }}m</span>
            </div>
            <div class="mem-stat" v-if="batteryInfo.time_to_full_mins && batteryInfo.is_charging">
              <span class="mem-stat-label">充满时间</span>
              <span class="mem-stat-value">{{ Math.floor(batteryInfo.time_to_full_mins / 60) }}h {{ Math.floor(batteryInfo.time_to_full_mins % 60) }}m</span>
            </div>
          </div>
        </div>
        <div v-else class="card glass">
          <p class="no-data">未检测到电池（台式机或不支持）</p>
        </div>
      </div>
    </main>
  </div>

  <!-- 加载画面 -->
  <div v-else class="loading-screen">
    <div class="loading-spinner"></div>
    <p>系统信息加载中...</p>
  </div>
</template>

<style>
* { margin: 0; padding: 0; box-sizing: border-box; }

body {
  background: linear-gradient(135deg, #0a0a1a 0%, #0f1628 30%, #151030 60%, #0a0a1a 100%);
  color: #e0e0e0;
  font-family: "Inter", "SF Pro Display", system-ui, -apple-system, sans-serif;
  overflow: hidden;
  height: 100vh;
}

/* ========== 布局 ========== */
.app-layout {
  display: flex;
  height: 100vh;
  overflow: hidden;
}

/* ========== 侧边栏 ========== */
.sidebar {
  width: 180px;
  min-width: 180px;
  background: rgba(15, 15, 30, 0.8);
  backdrop-filter: blur(20px);
  border-right: 1px solid rgba(255, 255, 255, 0.06);
  display: flex;
  flex-direction: column;
  padding: 16px 0;
}

.sidebar-title {
  font-size: 16px;
  font-weight: 700;
  padding: 0 16px 16px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  margin-bottom: 8px;
}

.nav-list {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
  padding: 0 8px;
}

.nav-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 12px;
  border: none;
  background: transparent;
  color: #888;
  font-size: 14px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
  font-family: inherit;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #bbb;
}

.nav-item.active {
  background: rgba(99, 102, 241, 0.15);
  color: #a5b4fc;
}

.nav-icon { font-size: 16px; }

.sidebar-footer {
  padding: 12px 16px;
  border-top: 1px solid rgba(255, 255, 255, 0.06);
  margin-top: 8px;
}

.host-name { font-size: 12px; color: #888; }
.os-name { font-size: 11px; color: #555; margin-top: 2px; }

/* ========== 内容区域 ========== */
.content {
  flex: 1;
  overflow-y: auto;
  padding: 24px;
}

.content::-webkit-scrollbar { width: 6px; }
.content::-webkit-scrollbar-track { background: transparent; }
.content::-webkit-scrollbar-thumb { background: rgba(255,255,255,0.1); border-radius: 3px; }

.page-title {
  font-size: 22px;
  font-weight: 700;
  margin-bottom: 20px;
}

/* ========== 毛玻璃卡片 ========== */
.card.glass {
  background: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(12px);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 14px;
  padding: 18px;
  margin-bottom: 14px;
  transition: border-color 0.2s;
}

.card.glass:hover {
  border-color: rgba(255, 255, 255, 0.12);
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 6px;
}

.card-icon { font-size: 18px; }

.card-label {
  font-size: 15px;
  font-weight: 600;
  flex: 1;
}

.card-sub {
  font-size: 13px;
  color: #666;
  margin-bottom: 10px;
}

.chip {
  font-size: 12px;
  font-weight: 700;
  padding: 2px 10px;
  border-radius: 20px;
  color: #000;
}

/* ========== 进度条 ========== */
.bar-bg {
  height: 8px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 4px;
  overflow: hidden;
}

.bar-bg.bar-sm { height: 5px; }
.bar-bg.bar-lg { height: 12px; border-radius: 6px; }

.bar-fill {
  height: 100%;
  border-radius: inherit;
  transition: width 0.6s ease, background 0.6s ease;
}

/* ========== 概览页网格 ========== */
.overview-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 14px;
}

.mini-chart {
  width: 100%;
  height: 40px;
  margin-top: 10px;
}

.disk-mini { margin-top: 10px; }
.disk-mini-header {
  display: flex;
  justify-content: space-between;
  font-size: 12px;
  color: #888;
  margin-bottom: 4px;
}

.net-mini { margin: 8px 0; }
.net-mini-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  margin-bottom: 4px;
}

.net-arrow { font-weight: 700; font-size: 16px; }
.net-arrow.down { color: #4ade80; }
.net-arrow.up { color: #60a5fa; }

.uptime-text {
  font-size: 20px;
  font-weight: 600;
  margin-top: 8px;
  color: #a5b4fc;
}

/* ========== 折线图 ========== */
.chart-container { position: relative; }

.chart {
  width: 100%;
  height: 150px;
  border-radius: 8px;
}

.chart-labels {
  position: absolute;
  top: 0;
  right: 4px;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  pointer-events: none;
}

.chart-labels span {
  font-size: 9px;
  color: #444;
}

/* ========== 核心网格 ========== */
.cores-grid {
  display: flex;
  gap: 8px;
  justify-content: center;
  flex-wrap: wrap;
  margin-top: 10px;
}

.core-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.core-bar-bg {
  width: 12px;
  height: 50px;
  background: rgba(255, 255, 255, 0.06);
  border-radius: 6px;
  overflow: hidden;
  display: flex;
  align-items: flex-end;
}

.core-bar-fill {
  width: 100%;
  border-radius: 6px;
  transition: height 0.5s ease, background 0.5s ease;
}

.core-label { font-size: 10px; color: #555; }
.core-value { font-size: 9px; color: #777; }

.section-title {
  font-size: 14px;
  font-weight: 600;
  margin-bottom: 12px;
  color: #999;
}

/* ========== 内存统计 ========== */
.mem-stats {
  display: flex;
  gap: 20px;
  margin-bottom: 14px;
}

.mem-stat { text-align: center; flex: 1; }
.mem-stat-label { display: block; font-size: 12px; color: #666; margin-bottom: 2px; }
.mem-stat-value { display: block; font-size: 16px; font-weight: 600; color: #ccc; }

/* ========== 网络页 ========== */
.highlight-card { border-color: rgba(99, 102, 241, 0.2) !important; }

.ip-info { text-align: center; margin: 12px 0; }
.ip-address { font-size: 28px; font-weight: 700; color: #a5b4fc; letter-spacing: 1px; }
.ip-details { font-size: 14px; color: #888; margin-top: 6px; }
.ip-org { font-size: 13px; color: #666; margin-top: 4px; }
.ip-loading { text-align: center; color: #666; padding: 20px; }

.refresh-btn {
  display: block;
  margin: 10px auto 0;
  padding: 6px 16px;
  background: rgba(99, 102, 241, 0.15);
  color: #a5b4fc;
  border: 1px solid rgba(99, 102, 241, 0.3);
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  font-family: inherit;
  transition: all 0.2s;
}

.refresh-btn:hover { background: rgba(99, 102, 241, 0.25); }

.speed-display {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 30px;
  margin: 14px 0;
}

.speed-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.speed-arrow {
  font-size: 24px;
  font-weight: 700;
}

.speed-arrow.down { color: #4ade80; }
.speed-arrow.up { color: #60a5fa; }

.speed-value { font-size: 18px; font-weight: 600; }
.speed-label { font-size: 12px; color: #666; }

.speed-divider {
  width: 1px;
  height: 40px;
  background: rgba(255, 255, 255, 0.1);
}

.mac-addr { font-size: 11px; color: #555; font-family: monospace; }

.net-details { margin-top: 8px; }

.net-ips {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-wrap: wrap;
  margin-bottom: 8px;
}

.net-detail-label { font-size: 12px; color: #666; }

.ip-tag {
  font-size: 12px;
  background: rgba(255, 255, 255, 0.05);
  padding: 2px 8px;
  border-radius: 4px;
  font-family: monospace;
  color: #aaa;
}

.net-traffic {
  display: flex;
  gap: 16px;
  font-size: 13px;
  color: #777;
}

/* ========== 进程表格 ========== */
.process-table {
  width: 100%;
  border-collapse: collapse;
}

.process-table th {
  text-align: left;
  font-size: 12px;
  color: #666;
  padding: 8px 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
  font-weight: 600;
}

.process-table td {
  padding: 8px 10px;
  font-size: 13px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.03);
}

.th-name, .td-name { width: 40%; }
.td-center { text-align: center; }
.td-name { color: #ccc; font-weight: 500; }

/* ========== 电池 ========== */
.battery-visual {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 10px 0;
}

.battery-icon-large {
  display: flex;
  align-items: center;
}

.battery-body {
  width: 120px;
  height: 50px;
  border: 2px solid #555;
  border-radius: 8px;
  overflow: hidden;
  display: flex;
  align-items: stretch;
}

.battery-level {
  height: 100%;
  transition: width 0.6s ease;
}

.battery-tip {
  width: 6px;
  height: 20px;
  background: #555;
  border-radius: 0 3px 3px 0;
  margin-left: 2px;
  align-self: center;
}

.battery-percent {
  font-size: 32px;
  font-weight: 700;
}

.battery-state {
  font-size: 14px;
  color: #888;
}

.no-data {
  text-align: center;
  color: #666;
  padding: 30px;
}

/* ========== 加载画面 ========== */
.loading-screen {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100vh;
  gap: 16px;
  color: #666;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: #a5b4fc;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}
</style>