<template>
  <n-card title="设置" style="max-width: 480px">
    <n-form label-placement="left" label-width="100">
      <n-form-item label="主题">
        <n-radio-group v-model:value="s.theme">
          <n-radio value="dark">深色</n-radio>
          <n-radio value="light">浅色</n-radio>
          <n-radio value="auto">跟随系统</n-radio>
        </n-radio-group>
      </n-form-item>
      <n-form-item label="默认视图">
        <n-radio-group v-model:value="s.defaultView">
          <n-radio value="tree">Tree</n-radio>
          <n-radio value="list">List</n-radio>
        </n-radio-group>
      </n-form-item>
    </n-form>
  </n-card>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { NCard, NForm, NFormItem, NRadioGroup, NRadio } from 'naive-ui';
import { useSettingsStore } from '../stores/settings';

const s = useSettingsStore();
onMounted(() => {
  if (!s.loaded) s.load();
});
</script>
