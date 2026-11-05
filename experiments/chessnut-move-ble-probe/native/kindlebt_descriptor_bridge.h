#ifndef PROBE_KINDLEBT_DESCRIPTOR_BRIDGE_H
#define PROBE_KINDLEBT_DESCRIPTOR_BRIDGE_H
/* Included in a generated copy of the exact pinned KindleBT translation unit.
 * Its other wrappers already forward through this same application table.
 * No borrowed descriptor/blob pointers are retained here. */
#include <kindlebt/kindlebt_application.h>
static void probe_descriptor_callback_bridge(bleConnHandle connection,
    bleGattCharacteristicsValue_t value,status_t status) {
    if(application_gatt_client_callbacks.on_ble_gattc_write_descriptor_cb!=NULL)
        application_gatt_client_callbacks.on_ble_gattc_write_descriptor_cb(connection,value,status);
}
#endif
