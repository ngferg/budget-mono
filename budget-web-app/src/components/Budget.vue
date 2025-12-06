<script setup>
import { ref, onMounted } from 'vue'
import { store } from '../store.js'
const error = ref("");
const budget = ref(null);
const categories = ref(null);

const get_budget = async () => {
  console.log("Get budget for: " + store.get_email());
  try {
    const resp = await fetch ('/api/users/budget', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        'email': store.get_email(),
        'year': 2025,
        'month': 12,
      })
    });
    if (resp.status === 200) {
      const j = await resp.json();
      console.log(j);
      budget.value = j.budget;
      categories.value = j.categories;
      console.log(budget.value);
      console.log(categories.value);
    } else {
      error.value = "Error: " + resp.status;
    }
  } catch (e) {
    error.value = "Error: " + resp.status;
  }
};

onMounted(async () => {
  await get_budget();
})
</script>

<template>
  <h1>Budget Page</h1>

  <div class="card">
    <p class="error-text">{{ error }}</p>
  </div>

</template>

<style scoped>
.error-text {
  color: #AA0000;
}
</style>
