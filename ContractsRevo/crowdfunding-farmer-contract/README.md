# Crowdfunding Farmer Contract

A smart contract for farmer crowdfunding on the Stellar network using Soroban.

## Features

- Farmers can create crowdfunding campaigns with funding goals and deadlines
- Contributors can fund campaigns with tokens
- Automatic reward distribution for successful campaigns (10% of total funded)
- Automatic refunds for failed campaigns
- Campaign status tracking (Active/Successful/Failed)

## Contract Functions

### Core Functions
- `create_campaign(farmer_id, goal_amount, deadline, reward_token)`  
  Creates a new campaign with specified parameters
- `contribute(contributor, campaign_id, amount)`  
  Contributes tokens to an active campaign
- `distribute_rewards(campaign_id)`  
  Distributes rewards for successful campaigns (callable after deadline)
- `refund_contributions(campaign_id)`  
  Refunds contributions for failed campaigns (callable after deadline)

### View Functions
- `get_campaign_details(campaign_id)`  
  Returns complete campaign details
- `get_contributions(campaign_id)`  
  Returns all contributions for a campaign

## Development

### Prerequisites
- Rust (latest stable version)
- Soroban CLI
- `wasm32-unknown-unknown` target

### Building
```bash
make build