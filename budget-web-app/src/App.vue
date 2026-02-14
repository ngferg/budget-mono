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
  <nav class="fixed top-0 left-0 right-0 z-50 w-screen bg-gray-800 py-1 shadow-md m-0">
    <div class="flex justify-between items-center px-8">
      <h1 class="text-white m-0 text-sm">febudget.com</h1>
      <button v-if="store.is_logged_in" @click="handleLogout"
        class="bg-red-500 text-white px-4 py-1 rounded cursor-pointer text-sm transition-colors hover:bg-red-600">
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
nav {
  background-color: #1f2937 !important;
  top: 0 !important;
  left: 0 !important;
}

:deep(body),
:deep(html) {
  margin: 0 !important;
  padding: 0 !important;
}
</style>
