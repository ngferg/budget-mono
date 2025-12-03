<script setup>
import { store } from './store.js'
import Login from './components/Login.vue'
import Budget from './components/Budget.vue'
import { ref } from 'vue'
</script>

<template>
  <Login v-if="!store.is_logged_in"/>
  <Budget v-if="store.is_logged_in" />
</template>

<style scoped>
.logo {
  height: 6em;
  padding: 1.5em;
  will-change: filter;
  transition: filter 300ms;
}
.logo:hover {
  filter: drop-shadow(0 0 2em #646cffaa);
}
.logo.vue:hover {
  filter: drop-shadow(0 0 2em #42b883aa);
}
</style>
