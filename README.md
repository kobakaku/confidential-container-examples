# Azure Confidential Container Examples

This repository contains example applications demonstrating Azure Confidential Container deployment with attestation capabilities.

## Overview

Azure Confidential Containers provide a secure environment for running containerized applications with hardware-based security guarantees. These examples showcase how to build and deploy applications that can access and display attestation reports.

## Examples

### 1. Hello World

A minimal web application that demonstrates:

- Attestation report retrieval and display

**Location**: `/hello-world`

### 2. GitHub Activity Verifier

A comprehensive web application for verifying GitHub user activity with TEE attestation:

- Real-time GitHub activity verification
- Multiple verification criteria (public repos, yearly commits, consecutive days, total stars)
- TEE environment attestation with MAA integration

**Location**: `/github-activity-verifier`

### 3. Confidential AI

Documentation and examples for confidential AI inference workflows:

- Sequence diagrams for confidential inference patterns
- Best practices for secure AI model deployment

**Location**: `/confidential-ai` (submodule, fork of [microsoft/confidential-ai](https://github.com/microsoft/confidential-ai))
