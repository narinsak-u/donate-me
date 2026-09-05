<script setup lang="ts">
import { computed } from 'vue'
import type { DayPoint } from '../api/dashboard'

const props = defineProps<{ data: DayPoint[]; goal: number }>()

// กราฟแท่ง SVG เบา ๆ — ไม่ต้องพึ่ง Chart.js
const max = computed(() => Math.max(...props.data.map((d) => d.total), 1))
const bars = computed(() =>
  props.data.map((d) => ({
    ...d,
    heightPct: (d.total / max.value) * 100,
    label: d.day.slice(8), // วันที่
    isToday: d.day === props.data[props.data.length - 1]?.day,
  })),
)

const goalPct = computed(() =>
  props.goal > 0 ? Math.min(100, Math.round((props.data.reduce((s, d) => s + d.total, 0) / props.goal) * 100)) : 0,
)
</script>

<template>
  <div>
    <div class="chart" v-if="data.length">
      <div
        v-for="b in bars"
        :key="b.day"
        class="bar-wrap"
        :title="`${b.day}: ฿${b.total.toLocaleString()}`"
      >
        <div class="bar" :class="{ today: b.isToday }" :style="{ height: `${Math.max(b.heightPct, 2)}%` }" />
        <span class="day">{{ b.label }}</span>
      </div>
    </div>

    <div v-if="goal > 0" class="goal">
      <div class="goal-bar">
        <div class="goal-fill" :style="{ width: `${goalPct}%` }" />
      </div>
      <span class="goal-text">เป้าหมาย ฿{{ goal.toLocaleString() }} — ผ่านแล้ว {{ goalPct }}%</span>
    </div>
  </div>
</template>

<style scoped>
.chart {
  display: flex;
  align-items: flex-end;
  gap: 2px;
  height: 140px;
  padding-top: 8px;
}
.bar-wrap {
  flex: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  align-items: center;
  position: relative;
}
.bar {
  width: 100%;
  max-width: 14px;
  background: linear-gradient(180deg, #7873f5, #ff6ec7);
  border-radius: 4px 4px 0 0;
  transition: height 0.3s;
}
.bar.today {
  background: linear-gradient(180deg, #ffe066, #ff6ec7);
}
.day {
  font-size: 9px;
  opacity: 0.4;
  margin-top: 4px;
}
.goal {
  margin-top: 20px;
}
.goal-bar {
  height: 12px;
  border-radius: 999px;
  background: rgba(0, 0, 0, 0.3);
  overflow: hidden;
}
.goal-fill {
  height: 100%;
  background: linear-gradient(90deg, #22c55e, #ffe066);
  border-radius: 999px;
  transition: width 0.5s;
}
.goal-text {
  font-size: 12px;
  opacity: 0.7;
  margin-top: 6px;
  display: block;
}
</style>
