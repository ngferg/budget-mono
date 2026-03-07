<script setup>

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
</style>
