#!/bin/bash

LOG=./target/release/corelog

echo "----------------------------------------"
echo "  Hyprcore Logging Demo"
echo "----------------------------------------"
echo ""

$LOG demo_rich
echo ""
$LOG boot_ok
$LOG net_up
$LOG db_connect
$LOG install_ok
echo ""
$LOG warn "Disk space low"
$LOG error "Database connection lost"
echo ""
$LOG test_rich
echo ""
echo "----------------------------------------"
