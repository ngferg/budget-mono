<script setup>
import { ref } from 'vue'
import { store } from '../store.js'

const email = ref("");
const error = ref("");

const login = async () => {
  console.log("Log in as: " + email.value);
  try {
    const resp = await fetch ('/api/users', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        'email': email.value,
      })
    });
    if (resp.status === 201 || resp.status === 409) {
      store.log_in_as(email.value);
    } else {
      error.value = "Error: " + resp.status;
    }
  } catch (e) {
    error.value = "Error: " + resp.status;
  }
};
</script>

<template>
  <h1>Login</h1>

  <div class="card">
    <input type="text" placeholder="email" v-model.trim="email" @keyup.enter="login" />
    <p class="error-text">{{ error }}</p>
  </div>

</template>

<style scoped>
.error-text {
  color: #AA0000;
}
</style>
