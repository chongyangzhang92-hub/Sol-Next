FROM python:3.12-slim

WORKDIR /app
COPY scripts/sol-cli.py /app/scripts/sol-cli.py
COPY mcp/sol_mcp_server.py /app/mcp/sol_mcp_server.py

ENV PYTHONUNBUFFERED=1
ENTRYPOINT ["python3", "/app/mcp/sol_mcp_server.py"]
