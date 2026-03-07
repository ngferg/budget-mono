<script setup>
import { store } from './store.js'
import Landing from './components/Landing.vue'
import Login from './components/Login.vue'
import Budget from './components/Budget.vue'
import Resources from './components/Resources.vue'
import { ref } from 'vue'

// Determine initial view
function initialView() {
  if (store.is_logged_in) return 'app';
  if (store.has_ever_logged_in()) return 'login';
  return 'landing';
}

const view = ref(initialView());
const show_resources = ref(false);

const handleLogout = async () => {
  await store.log_out();
  view.value = 'login';
}

const handleResources = () => {
  show_resources.value = true;
}

const handleBudget = () => {
  show_resources.value = false;
}

const handleLoginSuccess = () => {
  view.value = 'app';
}

const goToLogin = () => {
  view.value = 'login';
}

const goToLanding = () => {
  view.value = 'landing';
}
</script>

<template>
  <nav class="fixed top-0 left-0 right-0 z-50 w-screen bg-gray-800 py-1 shadow-md m-0">
    <div class="flex justify-between items-center px-8">
      <h1 class="text-white m-0 text-sm"><img src="/fe.png" class="h-16 w-16 mr-2 rounded" alt="febudget.com" /></h1>
      <div class="flex gap-4 items-center">
        <!-- Landing page -->
        <button v-if="view === 'landing'" @click="goToLogin"
          class="bg-white text-green-800 px-4 py-1 rounded cursor-pointer text-sm font-semibold transition-colors hover:bg-gray-100">
          Login
        </button>
        <!-- Login page -->
        <button v-if="view === 'login'" @click="goToLanding"
          class="text-white px-4 py-1 rounded cursor-pointer text-sm transition-colors hover:bg-white/10">
          About
        </button>
        <!-- Logged-in app -->
        <button v-if="view === 'app' && !show_resources" @click="handleResources">Resources</button>
        <button v-if="view === 'app' && show_resources" @click="handleBudget">Budget</button>
        <button v-if="view === 'app'" @click="handleLogout"
          class="bg-red-500 text-white px-4 py-1 rounded cursor-pointer text-sm transition-colors hover:bg-red-600">
          Log Out
        </button>
      </div>
    </div>
  </nav>
  <div class="mt-24">
    <Landing v-if="view === 'landing'" />
    <Login v-if="view === 'login'" @logged-in="handleLoginSuccess" />
    <Budget v-if="view === 'app' && !show_resources" />
    <Resources v-if="view === 'app' && show_resources" />
  </div>
</template>

<style scoped>
nav {
  background: linear-gradient(135deg, #065f46 0%, #047857 100%) !important;
  top: 0 !important;
  left: 0 !important;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3) !important;
}

:deep(body),
:deep(html) {
  margin: 0 !important;
  padding: 0 !important;
}
</style>
