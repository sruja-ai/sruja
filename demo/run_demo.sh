#!/bin/bash
set -a
source ../.env
set +a
export SRUJA_LLM_PROVIDER=openrouter
source ../.env

echo -e "\n\033[1;36m=====================================================\033[0m"
echo -e "\033[1;36m         SRUJA ARCHITECTURE INTELLIGENCE DEMO        \033[0m"
echo -e "\033[1;36m=====================================================\033[0m"

echo -e "\n\033[1;33m[1] THE RULEBOOK (INTENT)\033[0m"
echo "We deliberately designed architecture.sruja to state that our"
echo "Frontend should never talk to the Database."
cat architecture.sruja | grep -v '^$'

echo -e "\n\033[1;33m[2] THE REALITY (CODE SCAN)\033[0m"
echo "Running Sruja's code analysis scanner over our 3 demo microservices (\`frontend.py\`, \`api_gateway.py\`, \`database.py\`) to parse their Abstract Syntax Trees and detect dependencies..."
echo -e "\033[0;32m> cargo run -p sruja-cli -- scan --output sruja.graph.json\033[0m"
cargo run -q -p sruja-cli -- scan --output sruja.graph.json
echo "Graph generated successfully."

echo -e "\n\033[1;33m[3] DETECTING DRIFT (CODE VS. INTENT)\033[0m"
echo "Let's ask Sruja to compare the codebase against our stated rules:"
echo -e "\033[0;32m> cargo run -p sruja-cli -- drift -a architecture.sruja\033[0m"
cargo run -q -p sruja-cli -- drift -a architecture.sruja

echo -e "\n\033[1;33m[4] RUNTIME INTELLIGENCE (WITH DISTRIBUTED TRACES)\033[0m"
echo "We have a traces.json spanning multiple repos showing Frontend calling ThirdPartyPaymentAPI."
echo "Let's merge runtime behavior into the CTO stakeholder report."
echo -e "\033[0;32m> cargo run -p sruja-cli -- analyze --view cto -t traces.json\033[0m"
cargo run -q -p sruja-cli -- analyze --view cto -t traces.json

echo -e "\n\033[1;33m[5] ARCHITECTURAL INTELLIGENCE (LLM EXPLAINABILITY)\033[0m"
echo -e "Let's ask Sruja's AI module to explain a specific design flaw in plain English:"
echo -e "> cargo run -p sruja-cli -- ai ask \"Why does the Frontend directly access the database instead of using the API Gateway? What are the risks of this coupling?\" --graph sruja.graph.json"
cargo run -q -p sruja-cli -- ai ask "Why does the Frontend directly access the database instead of using the API Gateway? What are the risks of this coupling?" --graph sruja.graph.json

echo -e "\n\033[1;36m=====================================================\033[0m"
echo -e "\033[1;36m                     DEMO COMPLETE                   \033[0m"
echo -e "\033[1;36m=====================================================\033[0m"
