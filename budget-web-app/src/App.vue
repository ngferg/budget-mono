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
  <nav class="navbar">
    <div class="navbar-content">
      <h1>febudget.com</h1>
      <button v-if="store.is_logged_in" @click="handleLogout" class="logout-btn">
        Log Out
      </button>
    </div>
  </nav>
  <div class="main-content">
    <Login v-if="!store.is_logged_in" />
    <Budget v-if="store.is_logged_in" />
  </div>
</template>

<style scoped>
.navbar {
  background-color: #333;
  padding: 1rem 2rem;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  z-index: 100;
}

.navbar-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  max-width: 100%;
}

.navbar h1 {
  color: white;
  margin: 0;
  font-size: 1.5rem;
}

.main-content {
  margin-top: 70px;
}

.logout-btn {
  background-color: #ff6b6b;
  color: white;
  border: none;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  cursor: pointer;
  font-size: 1rem;
  transition: background-color 0.3s;
}

.logout-btn:hover {
  background-color: #ff5252;
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
