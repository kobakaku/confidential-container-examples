#!/bin/bash

# CCE Policy Generator Script for github-activity-verifier

set -e

echo "🔧 Generating CCE Policy for github-activity-verifier..."

if [ -z "$ACR_FQDN" ] || [ -z "$RESOURCE_GROUP" ]; then
    echo "❌ Error: Please set environment variables first:"
    echo "export ACR_FQDN=\"your-acr.azurecr.io\""
    echo "export RESOURCE_GROUP=\"your-resource-group\""
    echo ""
    echo "💡 You can also pass the image name as an argument:"
    echo "$0 <container-image-name>"
    exit 1
fi

# コンテナイメージの設定
if [ -n "$1" ]; then
    CONTAINER_IMAGE="$1"
elif [ -n "$ACR_FQDN" ]; then
    CONTAINER_IMAGE="$ACR_FQDN/github-activity-verifier:latest"
else
    echo "❌ Error: No container image specified"
    echo "💡 Usage: $0 <container-image-name>"
    echo "💡 Or set ACR_FQDN environment variable"
    exit 1
fi

echo "🔍 Using container image: $CONTAINER_IMAGE"

if ! az extension show --name confcom &> /dev/null; then
    echo "📦 Installing Azure CLI confcom extension..."
    az extension add --name confcom
fi

echo "📝 Generating policy from template..."
export CONTAINER_IMAGE
envsubst < policy-template.json > /tmp/policy-input.json

echo "🔍 Generated policy input:"
cat /tmp/policy-input.json | jq .

# Generate CCE policy   
echo "🏗️  Generating CCE policy..."
CCE_POLICY=$(az confcom acipolicygen \
    --input /tmp/policy-input.json)

echo "$CCE_POLICY" > policy.rego

echo "✅ CCE policy saved to: policy.rego"

# Set CCE_POLICY environment variable
echo "🔧 Setting CCE_POLICY environment variable..."
export CCE_POLICY

echo "🔧 Generating final parameters.json from template..."
# Generate final parameters.json from parameters-template.json
envsubst '$LOCATION,$ACR_FQDN,$MAA_ENDPOINT,$ACR_USERNAME,$ACR_PASSWORD,$CCE_POLICY' < parameters-template.json > parameters.json

echo "✅ parameters.json generated with all environment variables!"
echo ""
echo "📁 Generated files:"
echo "  - policy.rego (CCE policy)"
echo "  - parameters.json (with all environment variables expanded)"
echo ""
echo "🚀 Ready to deploy! The parameters file now contains the correct CCE policy."
echo ""
echo "💡 To deploy, run:"
echo "az deployment group create \\"
echo "  --resource-group \$RESOURCE_GROUP \\"
echo "  --template-file arm-template.json \\"
echo "  --parameters @parameters.json"

# Clean up temporary files
rm -f /tmp/policy-input.json

echo ""
echo "🎯 CCE Policy generation completed successfully!"