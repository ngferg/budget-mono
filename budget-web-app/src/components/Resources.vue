<script setup>
import { ref } from 'vue'
import { store } from '../store.js'
import { sha256 } from '../hash.js'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

const deleteError = ref('');
const deleteConfirm = ref(false);
const deleting = ref(false);

const deleteAccount = async () => {
    if (!deleteConfirm.value) {
        deleteConfirm.value = true;
        return;
    }
    deleting.value = true;
    deleteError.value = '';
    try {
        const resp = await fetch(API_BASE_URL + '/users', {
            method: 'DELETE',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': store.get_token(),
            },
            body: JSON.stringify({ hashed_email: await sha256(store.get_email()) }),
        });
        if (resp.status === 204) {
            await store.log_out();
            window.location.reload();
        } else {
            deleteError.value = `Failed to delete account (status ${resp.status}). Please try again.`;
            deleteConfirm.value = false;
        }
    } catch (e) {
        deleteError.value = 'An error occurred: ' + e.message;
        deleteConfirm.value = false;
    } finally {
        deleting.value = false;
    }
};

const cancelDelete = () => {
    deleteConfirm.value = false;
    deleteError.value = '';
};
</script>

<template>
    <div class="card">
        <h2>Resources</h2>
        <p>So you finished your budget for the month, and now you want to explore some additional resources to help you
            manage your finances better. Below are some useful links and tools to guide you on your financial journey,
            and help you save some money along the way.</p>
        <h3 class="text-2xl font-bold text-emerald-300 mt-6 mb-4 border-b-2 border-emerald-500 pb-2">Financial Tools:
        </h3>
        <ul>
            <li>
                <a href="https://www.chime.com/r/nicholausferguson/" target="_blank">
                    <span class="link-title">Chime</span>: Chime is a nice, modern
                    banking app that
                    offers fee-free checking and savings accounts, along with a variety of financial tools to help you
                    manage your money. The savings account also has competitive interest rates. When you open an account
                    with this link, and set up a minimum $200 direct deposit,
                    you will receive a cash bonus.
                </a>
            </li>
            <li>
                <a href="https://i.capitalone.com/GXIZYUJcE" target="_blank">
                    <span class="link-title">Capital One QuickSilver</span>: Capital One
                    QuickSilver is a
                    credit card that offers 1.5% cashback rewards on every purchase. On the MasterCard network, and has
                    no annual fee.
                </a>
            </li>
            <li>
                <a href="https://www.reliant.com/en/private/residential/deeplink/referrees-customer-referral?txtReferralID=LO1HYNL"
                    target="_blank">
                    <span class="link-title">Reliant Energy</span>: Live in Texas? Sign up for Reliant Energy using this
                    link to receive a bill credit when you sign up for service.
                </a>
            </li>
        </ul>

        <h3 class="text-2xl font-bold text-emerald-300 mt-6 mb-4 border-b-2 border-emerald-500 pb-2">Account
            Management:
        </h3>
        <div class="account-management-section">
            <h4 class="section-title">Delete Your Account</h4>
            <p class="section-desc">Permanently delete your account and all associated budget data. This action cannot
                be undone.</p>
            <p v-if="deleteError" class="delete-error">{{ deleteError }}</p>
            <div v-if="!deleteConfirm">
                <button class="delete-btn" @click="deleteAccount" :disabled="deleting">
                    Delete Account
                </button>
            </div>
            <div v-else class="confirm-row">
                <p class="confirm-text">Are you sure? This will permanently erase all your data.</p>
                <button class="delete-btn confirm" @click="deleteAccount" :disabled="deleting">
                    {{ deleting ? 'Deleting...' : 'Yes, Delete My Account' }}
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
h2 {
    color: #d1fae5;
    font-size: 1.5em;
    margin-bottom: 1em;
    text-align: left;
}

p {
    text-align: left;
    color: #d1fae5;
    line-height: 1.6;
    margin-bottom: 1.5em;
}

ul {
    list-style: none;
    padding: 0;
    margin: 0.8em 0;
}

li {
    text-align: left;
    margin: 0.8em 0;
    background: rgba(16, 185, 129, 0.08);
    border-radius: 8px;
    border-left: 3px solid #10b981;
    transition: all 0.2s ease;
    line-height: 1.6;
    cursor: pointer;
}

li:hover {
    background: rgba(16, 185, 129, 0.12);
    border-left-color: #34d399;
    transform: translateX(2px);
}

a {
    color: #d1fae5;
    text-decoration: none;
    transition: color 0.2s ease;
    display: block;
    padding: 1em 1.2em;
}

.link-title {
    color: #34d399;
    font-weight: 600;
}

li:hover .link-title {
    color: #6ee7b7;
    text-decoration: underline;
}

.account-management-section {
    background: rgba(239, 68, 68, 0.06);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
    padding: 1.2em 1.4em;
    margin-top: 0.5em;
}

.section-title {
    color: #fca5a5;
    font-size: 1.1em;
    font-weight: 600;
    margin-bottom: 0.5em;
    text-align: left;
}

.section-desc {
    color: #d1fae5;
    font-size: 0.95em;
    margin-bottom: 1em;
    text-align: left;
}

.delete-btn {
    background: rgba(239, 68, 68, 0.15);
    color: #fca5a5;
    border: 1px solid rgba(239, 68, 68, 0.5);
    border-radius: 6px;
    padding: 0.5em 1.2em;
    font-size: 0.95em;
    cursor: pointer;
    transition: all 0.2s ease;
}

.delete-btn:hover:not(:disabled) {
    background: rgba(239, 68, 68, 0.28);
    border-color: #ef4444;
    color: #fee2e2;
}

.delete-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
}

.delete-btn.confirm {
    background: rgba(239, 68, 68, 0.25);
}

.cancel-btn {
    background: rgba(16, 185, 129, 0.1);
    color: #d1fae5;
    border: 1px solid rgba(16, 185, 129, 0.4);
    border-radius: 6px;
    padding: 0.5em 1.2em;
    font-size: 0.95em;
    cursor: pointer;
    transition: all 0.2s ease;
    margin-left: 0.6em;
}

.cancel-btn:hover:not(:disabled) {
    background: rgba(16, 185, 129, 0.2);
}

.confirm-row {
    display: flex;
    flex-direction: column;
    gap: 0.6em;
    align-items: flex-start;
}

.confirm-text {
    color: #fca5a5;
    font-size: 0.9em;
    margin: 0;
    text-align: left;
}

.delete-error {
    color: #fca5a5;
    font-size: 0.9em;
    margin-bottom: 0.8em;
    text-align: left;
}
</style>
