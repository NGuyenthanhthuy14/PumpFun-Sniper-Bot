# Variables
CONFIG_PATH=/root/projects/my_bot/pump.fun-sniper-Rust/Config.toml
ASSETS_DIR_PATH=/root/projects/my_bot/pump.fun-sniper-Rust/src/assets
IMAGE=pumpfunbot

# Run sniper_mode container
sniper_mode:
	docker run -d \
		--name sniper_mode_container \
		-v $(CONFIG_PATH):/app/Config.toml \
		-e CONFIG_PATH=/app/Config.toml \
		$(IMAGE) sniper_mode

# Run monitor_mode container
monitor_mode:
	docker run -d \
		--name monitor_mode_container \
		-v $(CONFIG_PATH):/app/Config.toml \
		-v $(ASSETS_DIR_PATH):/app/src/assets \
		-e CONFIG_PATH=/app/Config.toml \
		$(IMAGE) monitor_mode

# Stop sniper_mode container
stop_sniper:
	docker stop sniper_mode_container || true
	docker rm sniper_mode_container || true

# Stop copy_mode container
stop_monitor:
	docker stop monitor_mode_container || true
	docker rm monitor_mode_container || true

#sniper log
sniper_log:
	docker logs -f sniper_mode_container

#copy log
monitor_log:
	docker logs -f monitor_mode_container
