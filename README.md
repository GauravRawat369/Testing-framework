# **Adaptive Payment Connector Routing Using Non-Stationary Multi-Armed Bandits**

## **Problem Statement**

In a payment system, we need to dynamically choose the best **payment connector** (e.g., Stripe, PayPal, Adyen, RazorPay, PayU) to maximize **successful transactions**. Each connector has a **success rate (SR)** that changes over time due to factors like:

- Network performance fluctuations
- Connector downtime
- Fraud detection policies
- Seasonal traffic variations

This makes the problem **non-stationary**, meaning past success rates may not reflect future performance.

## **Solution Approach: Non-Stationary Multi-Armed Bandits**
A **multi-armed bandit (MAB)** framework helps balance **exploration (trying different connectors)** and **exploitation (choosing the best one)**. Since success rates change over time, we use **non-stationary bandit algorithms** like:
- **Sliding Window UCB / Thompson Sampling** (keeps recent data only)
- **Discounted UCB / Thompson Sampling** (weights recent rewards more)

We focus on **Thompson Sampling with Discount Factor** and **Sliding Window UCB**, as they efficiently adapt to changes.

---

## **1. Thompson Sampling with Discount Factor**

### **Modeling the Problem Using Beta Distribution**
Each connector \( i \) has an **unknown success rate \( p_i \)**, modeled as a **Beta distribution**:
$$
p_i \sim \text{Beta}(\alpha_i, \beta_i)
$$
Where:
- \( \alpha_i \) → Number of **successful transactions** for connector \( i \)
- \( \beta_i \) → Number of **failed transactions** for connector \( i \)

### **Updating the Beta Distribution**
After each transaction:
- If **successful**:  
  $$
  \alpha_i \leftarrow \gamma \alpha_i + 1
  $$
- If **failed**:  
  $$
  \beta_i \leftarrow \gamma \beta_i + 1
  $$
- \( \gamma \) is a **discount factor** (0 < \( \gamma \) < 1) that reduces the weight of older data.

### **Choosing the Best Payment Connector (Thompson Sampling)**
1. **Sample a success rate** for each connector:
   $$
   \hat{p}_i \sim \text{Beta}(\alpha_i, \beta_i)
   $$
2. **Pick the connector with the highest sampled value**.

This ensures that:
- **High-success connectors get chosen more often**.
- **Uncertain connectors get explored occasionally**.

---

## **2. Sliding Window UCB (Upper Confidence Bound)**

### **Sliding Window for Success Rate Estimation**
A sliding window of size \( W \) is used to track the most recent transaction outcomes for each connector. The success rate is estimated as:
$$
\text{Success Rate}_i = \frac{\text{Number of Successes in Window}}{W}
$$

### **UCB Score Calculation**
The UCB score for each connector is computed as:
$$
\text{UCB Score}_i = \text{Success Rate}_i + c \cdot \sqrt{\frac{\ln(\text{Total Attempts})}{\text{Connector Attempts}_i}}
$$
Where:
- \( c \) is an **exploration factor**.
- \( \text{Total Attempts} \) is the total number of transactions across all connectors.
- \( \text{Connector Attempts}_i \) is the number of transactions routed through connector \( i \).

### **Connector Selection**
The connector with the highest UCB score is selected for the next transaction:
$$
g^* = \arg\max_{g} \left( \text{Success Rate}_g + c \cdot \sqrt{\frac{\ln(\text{Total Attempts})}{\text{Connector Attempts}_g}} \right)
$$

---

## **Choosing the Discount Factor (\( \gamma \))**
| Discount Factor \( \gamma \) | Adaptation Speed | Best Use Case |
|------------------|---------------------|------------------------------|
| **1.0** | No forgetting | Use when success rates are mostly stable |
| **0.99 - 0.95** | Slow forgetting | Good for gradual changes over days/weeks |
| **0.9 - 0.8** | Fast forgetting | Works well for hourly/daily changes |
| **< 0.7** | Very aggressive forgetting | Use for highly volatile environments (rare) |

---

## **Comparison of Methods**

| Method                          | Exploration & Exploitation | Handles Failures | Adaptability to Non-Stationarity | Complexity |
|---------------------------------|----------------------------|------------------|----------------------------------|------------|
| Thompson Sampling with Discount | High                      | No               | High                             | High       |
| Sliding Window UCB              | Medium                    | No               | Medium                           | Low        |

---

## **Final Takeaway**
By using **Thompson Sampling with Discount Factor** or **Sliding Window UCB**, we can dynamically select the best payment connector, adapting to **changing success rates** in real-time. These methods efficiently balance **exploration vs. exploitation** while ensuring optimal transaction success.

- **Thompson Sampling with Discount Factor** is ideal for environments where exploration is critical, and connector performance changes gradually.
- **Sliding Window UCB** is a simple and efficient solution for moderately non-stationary environments.

---

## **How to Run the Code**

1. Open a terminal in the project root directory.
2. Build and run using Cargo:
   - For Thompson Sampling:
     ```bash
     cargo run -- thompson
     ```
   - For Sliding Window UCB:
     ```bash
     cargo run -- ucb
     ```

---

