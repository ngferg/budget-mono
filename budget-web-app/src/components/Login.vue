<script setup>
import { ref, nextTick } from 'vue'
import { store } from '../store.js'

const AUTH_BASE_URL = import.meta.env.VITE_AUTH_BASE_URL || 'http://localhost:3001';
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';
const email = ref("");
const error = ref("");
const code = ref("");
const token = ref("");
const code_requested = ref(false);
const code_input = ref(null);

const request_code = async () => {
  console.log("Request code for: " + email.value);
  try {
    const resp = await fetch(AUTH_BASE_URL + '/request_code', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        'email': email.value,
      })
    });
    if (resp.status === 200) {
      code_requested.value = true;
      nextTick(() => {
        code_input.value.focus();
      });
    } else {
      error.value = "Error: " + resp.status;
    }
  } catch (e) {
    error.value = "Error: " + e.message;
  }
};

const verify_code = async () => {
  console.log("Verify code for: " + email.value);
  try {
    const resp = await fetch(AUTH_BASE_URL + '/verify_code', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        'email': email.value,
        'code': code.value,
      })
    });
    console.log(resp);
    if (resp.status === 200) {
      const data = await resp.json();
      token.value = data.token;
      await login();
    } else {
      error.value = "Incorrect code";
    }
  } catch (e) {
    error.value = "Error: " + e.message;
  }
};

const login = async () => {
  console.log("Log in as: " + email.value);
  try {
    const resp = await fetch(API_BASE_URL + '/users', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': token.value,
      },
      body: JSON.stringify({
        'email': email.value,
      })
    });
    if (resp.status === 201 || resp.status === 409) {
      store.log_in_as(email.value, token.value);
    } else {
      error.value = "Login failed";
    }
  } catch (e) {
    error.value = "Error: " + e.message;
  }
};
</script>

<template>
  <h1>Login</h1>

  <div class="card">
    <div><input type="text" placeholder="email" v-model.trim="email" @keyup.enter="request_code" /></div>
    <div><input type="text" placeholder="code" v-model.trim="code" @keyup.enter="verify_code" v-if="code_requested"
        ref="code_input" />
    </div>
    <p class="error-text">{{ error }}</p>
  </div>

</template>

<style scoped>
.error-text {
  color: #AA0000;
}
</style>
