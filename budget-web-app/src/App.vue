<script setup>
import { store } from './store.js'
import Login from './components/Login.vue'
import Budget from './components/Budget.vue'
import { ref } from 'vue'

const handleLogout = () => {
  store.log_out();
  location.reload();
}
</script>

<template>
  <nav class="fixed top-0 left-0 right-0 z-50 w-full bg-gray-800 px-8 py-4 shadow-md navbar">
    <div class="flex justify-between items-center">
      <h1 class="text-white m-0 text-m">febudget.com</h1>
      <button v-if="store.is_logged_in" @click="handleLogout"
        class="bg-red-500 text-white px-4 py-2 rounded cursor-pointer text-base transition-colors hover:bg-red-600">
        Log Out
      </button>
    </div>
  </nav>
  <div class="mt-24">
    <Login v-if="!store.is_logged_in" />
    <Budget v-if="store.is_logged_in" />
  </div>
</template>

<style scoped>
.navbar {
  position: fixed !important;
  top: 0 !important;
  left: 0 !important;
  right: 0 !important;
  background-color: #1f2937 !important;
  z-index: 50 !important;
}

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
