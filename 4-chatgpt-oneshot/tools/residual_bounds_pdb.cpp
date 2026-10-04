#define main campaign_search_main
#include "campaign_exact.cpp"
#undef main
#include <fstream>

int main(int argc, char** argv) {
    if (argc < 2 || argc > 4) return 1;
    int n = argc >= 3 ? std::stoi(argv[2]) : 8;
    if (n < 1 || n > 8) return 1;
    Database database(n);
    std::ofstream output(argv[1], std::ios::binary);
    if (argc == 4 && std::string(argv[3]) == "residual") {
        uint64_t count = database.factorial.at(n) * database.cuts.size();
        for (uint64_t id = 0; id < count; ++id)
            output.put(residual_bound(database.state(id), n));
    } else {
        build(database);
        for (uint8_t distance : database.distances) output.put(distance);
    }
    return output.good() ? 0 : 1;
}
