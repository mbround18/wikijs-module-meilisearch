# Kubernetes Deployment Guide: Wiki.js + Meilisearch + Meilisearch Module

This guide explains how to deploy Wiki.js, Meilisearch, and the Meilisearch search module on Kubernetes using best practices for modularity and security.

## Prerequisites

- A running Kubernetes cluster
- `kubectl` access
- Persistent storage provisioner (for Wiki.js and Meilisearch data)

---

## 1. Persistent Volume Claims

Define PVCs for Wiki.js data and Meilisearch data, and for the Meilisearch module (optional, for upgrades):

```yaml
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: wikijs-data
spec:
  accessModes: [ReadWriteOnce]
  resources:
    requests:
      storage: 5Gi
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: meilisearch-data
spec:
  accessModes: [ReadWriteOnce]
  resources:
    requests:
      storage: 5Gi
---
apiVersion: v1
kind: PersistentVolumeClaim
metadata:
  name: meilisearch-module
spec:
  accessModes: [ReadWriteOnce]
  resources:
    requests:
      storage: 1Gi
```

---

## 2. Meilisearch Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: meilisearch
spec:
  replicas: 1
  selector:
    matchLabels:
      app: meilisearch
  template:
    metadata:
      labels:
        app: meilisearch
    spec:
      containers:
        - name: meilisearch
          image: getmeili/meilisearch:v1.19
          ports:
            - containerPort: 7700
          env:
            - name: MEILI_MASTER_KEY
              value: "demo"
            - name: MEILI_NO_ANALYTICS
              value: "true"
            - name: MEILI_LOG_LEVEL
              value: "DEBUG"
          volumeMounts:
            - name: meili-data
              mountPath: /meili_data
      volumes:
        - name: meili-data
          persistentVolumeClaim:
            claimName: meilisearch-data
```

---

## 3. Wiki.js Deployment (with Meilisearch Module)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: wikijs
spec:
  replicas: 1
  selector:
    matchLabels:
      app: wikijs
  template:
    metadata:
      labels:
        app: wikijs
    spec:
      initContainers:
        - name: wikijs-meilisearch-module-init
          image: mbround18/wikijs-meilisearch-module:latest
          command:
            - sh
            - -c
          args:
            - cp -r /modules/meilisearch/* /wiki/server/modules/search/meilisearch && chown -R 1000:1000 /wiki/server/modules/search/meilisearch
          volumeMounts:
            - name: meilisearch-module
              mountPath: /wiki/server/modules/search/meilisearch
      containers:
        - name: wikijs
          image: requarks/wiki:2
          ports:
            - containerPort: 3000
          env:
            - name: DB_TYPE
              value: sqlite
          volumeMounts:
            - name: wikijs-data
              mountPath: /wiki/data
            - name: meilisearch-module
              mountPath: /wiki/server/modules/search/meilisearch
      volumes:
        - name: wikijs-data
          persistentVolumeClaim:
            claimName: wikijs-data
        - name: meilisearch-module
          persistentVolumeClaim:
            claimName: meilisearch-module
```

---

## 4. Accessing Wiki.js and Meilisearch

Expose services as needed (NodePort, LoadBalancer, or Ingress):

```yaml
apiVersion: v1
kind: Service
metadata:
  name: wikijs
spec:
  selector:
    app: wikijs
  ports:
    - port: 3000
      targetPort: 3000
---
apiVersion: v1
kind: Service
metadata:
  name: meilisearch
spec:
  selector:
    app: meilisearch
  ports:
    - port: 7700
      targetPort: 7700
```

---

## 5. Security & Best Practices

- Use a restricted Meilisearch API key for Wiki.js (see README for details)
- Use persistent storage for all data
- Use resource requests/limits for production
- Rotate module image for upgrades

---

## 6. Example: Minimal Working Setup

This is a minimal working example. Adjust namespaces, storage, and images as needed for your environment.
